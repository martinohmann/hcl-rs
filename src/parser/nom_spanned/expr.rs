use super::ast::{
    Array, BinaryOp, Conditional, Expression, ForCond, ForExpr, ForIntro, FuncCall, FuncSig,
    HeredocTemplate, Object, ObjectItem, ObjectKey, ObjectKeyValueSeparator, ObjectValueTerminator,
    Template, Traversal, TraversalOperator, UnaryOp,
};
use super::repr::{Decorate, Decorated, Span, Spanned};
use super::{
    anychar_except, char_or_cut, decor,
    error::InternalError,
    ident,
    input::Location,
    line_comment, number, prefix_decor, sp, span, spanned, str_ident, string, tag_or_cut,
    template::{string_template, template},
    with_span, ws, ErrorKind, IResult, Input,
};
use crate::expr::{BinaryOperator, UnaryOperator, Variable};
use crate::Identifier;
use nom::combinator::map_parser;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, crlf, line_ending, newline, space0, u64},
    combinator::{cut, fail, map, not, opt, peek, recognize, value},
    error::context,
    multi::{many1, many1_count, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};
use std::ops::Range;

fn array(input: Input) -> IResult<Input, Expression> {
    delimited(
        char('['),
        alt((
            map(for_list_expr, |expr| Expression::ForExpr(Box::new(expr))),
            map(array_items, |array| Expression::Array(Box::new(array))),
        )),
        char_or_cut(']'),
    )(input)
}

fn array_items(input: Input) -> IResult<Input, Array> {
    alt((
        map(
            pair(
                separated_list1(char(','), decor(ws, expr, ws)),
                opt(preceded(char(','), span(ws))),
            ),
            |(values, suffix_span)| {
                let mut array = Array::new(values);
                if let Some(suffix_span) = suffix_span {
                    array.set_trailing_comma(true);
                    array.set_trailing(suffix_span);
                }
                array
            },
        ),
        map(span(ws), |suffix_span| {
            let mut array = Array::default();
            array.set_trailing(suffix_span);
            array
        }),
    ))(input)
}

fn for_list_expr(input: Input) -> IResult<Input, ForExpr> {
    map(
        tuple((for_intro, decor(ws, cut(expr), ws), opt(for_cond))),
        |(intro, value_expr, cond)| {
            let mut expr = ForExpr::new(intro, value_expr);

            if let Some(cond) = cond {
                expr.set_cond(cond);
            }

            expr
        },
    )(input)
}

fn object(input: Input) -> IResult<Input, Expression> {
    delimited(
        char('{'),
        alt((
            map(for_object_expr, |expr| Expression::ForExpr(Box::new(expr))),
            map(object_items, |object| Expression::Object(Box::new(object))),
        )),
        char_or_cut('}'),
    )(input)
}

fn object_items(input: Input) -> IResult<Input, Object> {
    let mut remaining_input = input;
    let mut items = Vec::new();

    loop {
        let start = remaining_input.location();

        let (input, mut item) = match object_item(remaining_input) {
            Ok((input, item)) => (input, item),
            Err(nom::Err::Failure(err)) => return Err(nom::Err::Failure(err)),
            Err(err) => {
                // Consume all trailing whitespace and look for the closing brace, otherwise
                // propagate the error that occurred while parsing the object item.
                match terminated(span(ws), peek(char('}')))(remaining_input) {
                    Ok((input, suffix_span)) => {
                        let mut object = Object::new(items);
                        object.set_trailing(suffix_span);
                        return Ok((input, object));
                    }
                    Err(_) => return Err(err),
                }
            }
        };

        // Look for the closing brace and return or consume the object item separator and proceed
        // with the next object item, if any.
        let (input, ch) = peek(anychar)(input)?;

        let (input, value_terminator) = match ch {
            '}' => {
                item.set_span(start..input.location());
                items.push(item);
                return Ok((input, Object::new(items)));
            }
            '\r' => value(ObjectValueTerminator::Newline, crlf)(input)?,
            '\n' => value(ObjectValueTerminator::Newline, newline)(input)?,
            ',' => value(ObjectValueTerminator::Comma, char(','))(input)?,
            '#' | '/' => {
                let (input, comment_span) = span(line_comment)(input)?;

                // Associate the trailing comment with the item value, updating
                // the span if it already has a decor suffix.
                let suffix_start = match item.value().decor().suffix() {
                    Some(suffix) => suffix.span().unwrap().start,
                    None => comment_span.start,
                };

                item.value_mut()
                    .decor_mut()
                    .set_suffix(suffix_start..comment_span.end);

                value(ObjectValueTerminator::Newline, line_ending)(input)?
            }
            _ => {
                return Err(nom::Err::Failure(InternalError::new(
                    input,
                    ErrorKind::Context("closing brace, comma or newline"),
                )))
            }
        };

        item.set_span(start..input.location());
        item.set_value_terminator(value_terminator);
        items.push(item);
        remaining_input = input;
    }
}

fn object_key(input: Input) -> IResult<Input, ObjectKey> {
    decor(
        ws,
        map(expr, |expr| {
            // Variable identifiers without traversal are treated as identifier object keys.
            //
            // Handle this case here by converting the variable into an identifier. This
            // avoids re-parsing the whole key-value pair when an identifier followed by a
            // traversal operator is encountered.
            if let Expression::Variable(variable) = expr {
                ObjectKey::Identifier(Decorated::new(variable.into_inner().into()))
            } else {
                ObjectKey::Expression(expr)
            }
        }),
        sp,
    )(input)
}

fn object_key_value_separator(input: Input) -> IResult<Input, ObjectKeyValueSeparator> {
    alt((
        value(ObjectKeyValueSeparator::Equals, char('=')),
        value(ObjectKeyValueSeparator::Colon, char(':')),
    ))(input)
}

fn object_item(input: Input) -> IResult<Input, ObjectItem> {
    map(
        tuple((
            object_key,
            cut(object_key_value_separator),
            cut(decor(sp, expr, sp)),
        )),
        |(key, key_value_separator, value)| {
            let mut item = ObjectItem::new(key, value);
            item.set_key_value_separator(key_value_separator);
            item
        },
    )(input)
}

fn for_object_expr(input: Input) -> IResult<Input, ForExpr> {
    map(
        tuple((
            for_intro,
            separated_pair(
                decor(ws, cut(expr), ws),
                tag_or_cut("=>"),
                decor(ws, cut(expr), ws),
            ),
            opt(tag("...")),
            opt(for_cond),
        )),
        |(intro, (key_expr, value_expr), grouping, cond)| {
            let mut expr = ForExpr::new(intro, value_expr);
            expr.set_key_expr(key_expr);
            expr.set_grouping(grouping.is_some());

            if let Some(cond) = cond {
                expr.set_cond(cond);
            }

            expr
        },
    )(input)
}

fn for_intro(input: Input) -> IResult<Input, ForIntro> {
    prefix_decor(
        ws,
        map(
            delimited(
                tag("for"),
                tuple((
                    decor(ws, cut(ident), ws),
                    opt(preceded(char(','), decor(ws, cut(ident), ws))),
                    preceded(tag_or_cut("in"), decor(ws, cut(expr), ws)),
                )),
                char_or_cut(':'),
            ),
            |(first, second, expr)| match second {
                Some(second) => {
                    let mut intro = ForIntro::new(second, expr);
                    intro.set_key_var(first);
                    intro
                }
                None => ForIntro::new(first, expr),
            },
        ),
    )(input)
}

fn for_cond(input: Input) -> IResult<Input, ForCond> {
    prefix_decor(
        ws,
        map(preceded(tag("if"), decor(ws, cut(expr), ws)), ForCond::new),
    )(input)
}

fn parenthesis(input: Input) -> IResult<Input, Expression> {
    delimited(char('('), decor(ws, cut(expr), ws), char_or_cut(')'))(input)
}

fn heredoc_start(input: Input) -> IResult<Input, (bool, (&str, Range<usize>))> {
    terminated(
        pair(
            map(preceded(tag("<<"), opt(char('-'))), |indent| {
                indent.is_some()
            }),
            with_span(cut(str_ident)),
        ),
        cut(line_ending),
    )(input)
}

fn heredoc_content<'a>(delim: &'a str) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Template> {
    map_parser(
        recognize(pair(
            many1_count(anychar_except(tuple((line_ending, space0, tag(delim))))),
            line_ending,
        )),
        template,
    )
}

fn heredoc(input: Input) -> IResult<Input, HeredocTemplate> {
    let (input, (indented, (delim, delim_span))) = heredoc_start(input)?;

    let (input, (template, trailing)) = pair(
        spanned(map(opt(heredoc_content(delim)), Option::unwrap_or_default)),
        terminated(span(space0), cut(tag(delim))),
    )(input)?;

    let mut heredoc = HeredocTemplate::new(
        Decorated::new(Identifier::unchecked(delim)).spanned(delim_span),
        template,
    );

    if indented {
        heredoc.dedent();
    }

    heredoc.set_trailing(trailing);

    Ok((input, heredoc))
}

fn traversal_operator(input: Input) -> IResult<Input, TraversalOperator> {
    context(
        "TraversalOperator",
        alt((
            preceded(
                char('.'),
                prefix_decor(
                    ws,
                    preceded(
                        // Must not match `for` object value grouping or func call expand final which
                        // are both `...`.
                        not(char('.')),
                        cut(alt((
                            value(TraversalOperator::AttrSplat(Decorated::new(())), char('*')),
                            map(ident, |ident| TraversalOperator::GetAttr(ident.into())),
                            map(u64, |index| TraversalOperator::LegacyIndex(index.into())),
                        ))),
                    ),
                ),
            ),
            delimited(
                char('['),
                decor(
                    ws,
                    cut(alt((
                        value(TraversalOperator::FullSplat(Decorated::new(())), char('*')),
                        map(expr, TraversalOperator::Index),
                    ))),
                    ws,
                ),
                char_or_cut(']'),
            ),
        )),
    )(input)
}

fn ident_or_func_call(input: Input) -> IResult<Input, Expression> {
    map(
        pair(with_span(str_ident), opt(prefix_decor(ws, func_sig))),
        |((ident, span), signature)| match signature {
            Some(signature) => {
                let name = Decorated::new(Identifier::unchecked(ident)).spanned(span);
                let func_call = FuncCall::new(name, signature);
                Expression::FuncCall(Box::new(func_call))
            }
            None => match ident {
                "null" => Expression::Null(().into()),
                "true" => Expression::Bool(true.into()),
                "false" => Expression::Bool(false.into()),
                var => Expression::Variable(Variable::unchecked(var).into()),
            },
        },
    )(input)
}

fn func_sig(input: Input) -> IResult<Input, FuncSig> {
    delimited(
        char('('),
        alt((
            map(
                pair(
                    separated_list1(char(','), decor(ws, expr, ws)),
                    opt(pair(alt((tag(","), tag("..."))), span(ws))),
                ),
                |(args, trailer)| {
                    let mut sig = FuncSig::new(args);

                    if let Some((sep, trailing)) = trailer {
                        if sep.as_ref() == b"..." {
                            sig.set_expand_final(true);
                        } else {
                            sig.set_trailing_comma(true);
                        }

                        sig.set_trailing(trailing);
                    }

                    sig
                },
            ),
            map(span(ws), |trailing| {
                let mut sig = FuncSig::new(Vec::new());
                sig.set_trailing(trailing);
                sig
            }),
        )),
        char_or_cut(')'),
    )(input)
}

fn unary_operator(input: Input) -> IResult<Input, Spanned<UnaryOperator>> {
    map(
        alt((
            value(UnaryOperator::Neg, char('-')),
            value(UnaryOperator::Not, char('!')),
        )),
        Spanned::new,
    )(input)
}

fn binary_operator(input: Input) -> IResult<Input, BinaryOperator> {
    alt((
        value(BinaryOperator::Eq, tag("==")),
        value(BinaryOperator::NotEq, tag("!=")),
        value(BinaryOperator::LessEq, tag("<=")),
        value(BinaryOperator::GreaterEq, tag(">=")),
        value(BinaryOperator::Less, char('<')),
        value(BinaryOperator::Greater, char('>')),
        value(BinaryOperator::Plus, char('+')),
        value(BinaryOperator::Minus, char('-')),
        value(BinaryOperator::Mul, char('*')),
        value(BinaryOperator::Div, char('/')),
        value(BinaryOperator::Mod, char('%')),
        value(BinaryOperator::And, tag("&&")),
        value(BinaryOperator::Or, tag("||")),
    ))(input)
}

fn unary_op(input: Input) -> IResult<Input, Expression> {
    map(
        pair(spanned(unary_operator), prefix_decor(sp, expr_term)),
        |(operator, expr)| Expression::UnaryOp(Box::new(UnaryOp::new(operator, expr))),
    )(input)
}

fn expr_term(input: Input) -> IResult<Input, Expression> {
    let (input, ch) = peek(anychar)(input)?;

    match ch {
        '"' => alt((
            map(string, |s| Expression::String(s.into())),
            map(string_template, Expression::Template),
        ))(input),
        '[' => array(input),
        '{' => object(input),
        '0'..='9' => map(number, |n| Expression::Number(n.into()))(input),
        '<' => map(heredoc, |heredoc| {
            Expression::HeredocTemplate(Box::new(heredoc))
        })(input),
        '-' => alt((
            map(preceded(pair(char('-'), sp), number), |n| {
                Expression::Number((-n).into())
            }),
            unary_op,
        ))(input),
        '!' => unary_op(input),
        '(' => map(parenthesis, |expr| {
            Expression::Parenthesis(Box::new(expr.into()))
        })(input),
        _ => alt((ident_or_func_call, fail))(input),
    }
}

pub fn expr_inner(input: Input) -> IResult<Input, Expression> {
    let traversal = with_span(many1(prefix_decor(sp, traversal_operator)));

    let binary_op = with_span(pair(
        prefix_decor(sp, binary_operator),
        prefix_decor(sp, cut(expr)),
    ));

    let conditional = tuple((
        span(sp),
        preceded(char('?'), decor(sp, cut(expr), sp)),
        preceded(char_or_cut(':'), prefix_decor(sp, cut(expr))),
    ));

    map(
        tuple((
            with_span(expr_term),
            opt(traversal),
            opt(binary_op),
            opt(conditional),
        )),
        |((mut expr, span), traversal, binary_op, conditional)| {
            let start = span.start;
            let end = span.end;

            let (mut expr, end) = match traversal {
                Some((operators, span)) => {
                    expr.set_span(start..end);

                    let expr = Expression::Traversal(Box::new(Traversal::new(expr, operators)));

                    (expr, span.end)
                }
                None => (expr, end),
            };

            let (mut expr, end) = match binary_op {
                Some(((operator, rhs_expr), span)) => {
                    expr.set_span(start..end);

                    let expr =
                        Expression::BinaryOp(Box::new(BinaryOp::new(expr, operator, rhs_expr)));

                    (expr, span.end)
                }
                None => (expr, end),
            };

            match conditional {
                Some((suffix_span, true_expr, false_expr)) => {
                    // Associate whitespace preceding the `?` with the cond expression, updating
                    // the span if it already has a decor suffix.
                    let suffix_start = match expr.decor().suffix() {
                        Some(suffix) => suffix.span().unwrap().start,
                        None => suffix_span.start,
                    };

                    expr.decor_mut().set_suffix(suffix_start..suffix_span.end);
                    expr.set_span(start..end);

                    Expression::Conditional(Box::new(Conditional::new(expr, true_expr, false_expr)))
                }
                None => expr,
            }
        },
    )(input)
}

pub fn expr(input: Input) -> IResult<Input, Expression> {
    context("Expression", expr_inner)(input)
}
