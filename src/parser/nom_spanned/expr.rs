use super::ast::{
    Array, BinaryOp, Conditional, Expression, ForExpr, FuncCall, HeredocTemplate, Object,
    ObjectItem, ObjectKey, ObjectKeyValueSeparator, ObjectValueTerminator, Operation, Template,
    Traversal, TraversalOperator, UnaryOp,
};
use super::{
    anychar_except, char_or_cut, decorated,
    error::InternalError,
    ident, line_comment, number, prefix_decorated,
    repr::{Decor, Formatted},
    sp, span, spanned, str_ident, string, tag_or_cut,
    template::{heredoc_template, quoted_string_template},
    with_decor, with_span, ws, ErrorKind, IResult, Input,
};
use crate::Identifier;
use crate::{
    expr::{BinaryOperator, HeredocStripMode, UnaryOperator, Variable},
    util::dedent,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, crlf, line_ending, newline, space0, u64},
    combinator::{cut, fail, map, map_res, not, opt, peek, recognize, value},
    error::context,
    multi::{many1, many1_count, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};
use std::borrow::Cow;
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
                separated_list1(char(','), decorated(ws, expr, ws)),
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
        tuple((
            preceded(ws, for_intro),
            decorated(ws, cut(expr), ws),
            opt(for_cond_expr),
        )),
        |(intro, value_expr, cond_expr)| ForExpr {
            key_var: intro.key_var,
            value_var: intro.value_var,
            collection_expr: intro.collection_expr,
            key_expr: None,
            value_expr,
            cond_expr,
            grouping: false,
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
            Ok(res) => res,
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
                item.decor_mut()
                    .set_suffix(comment_span.start..comment_span.end);
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
    map(
        with_decor(ws, with_span(expr), sp),
        |((value, span), decor)| {
            // Variable identifiers without traversal are treated as identifier object keys.
            //
            // Handle this case here by converting the variable into an identifier. This
            // avoids re-parsing the whole key-value pair when an identifier followed by a
            // traversal operator is encountered.
            if let Expression::Variable(variable) = value {
                ObjectKey::Identifier(Formatted::new_with_decor(
                    variable.into_inner(),
                    span,
                    decor,
                ))
            } else {
                ObjectKey::Expression(Formatted::new_with_decor(value, span, decor))
            }
        },
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
            cut(decorated(sp, expr, sp)),
        )),
        |(key, key_value_separator, value)| ObjectItem {
            key,
            key_value_separator,
            value,
            value_terminator: ObjectValueTerminator::None,
            span: None,
            decor: Decor::default(),
        },
    )(input)
}

fn for_object_expr(input: Input) -> IResult<Input, ForExpr> {
    map(
        tuple((
            preceded(ws, for_intro),
            separated_pair(
                decorated(ws, cut(expr), ws),
                tag_or_cut("=>"),
                decorated(ws, cut(expr), ws),
            ),
            opt(terminated(tag("..."), ws)),
            opt(for_cond_expr),
        )),
        |(intro, (key_expr, value_expr), grouping, cond_expr)| ForExpr {
            key_var: intro.key_var,
            value_var: intro.value_var,
            collection_expr: intro.collection_expr,
            key_expr: Some(key_expr),
            value_expr,
            cond_expr,
            grouping: grouping.is_some(),
        },
    )(input)
}

struct ForIntro {
    key_var: Option<Formatted<Identifier>>,
    value_var: Formatted<Identifier>,
    collection_expr: Formatted<Expression>,
}

fn for_intro(input: Input) -> IResult<Input, ForIntro> {
    map(
        delimited(
            tag("for"),
            tuple((
                decorated(ws, cut(ident), ws),
                opt(preceded(char(','), decorated(ws, cut(ident), ws))),
                preceded(tag_or_cut("in"), decorated(ws, cut(expr), ws)),
            )),
            char_or_cut(':'),
        ),
        |(first, second, expr)| match second {
            Some(second) => ForIntro {
                key_var: Some(first),
                value_var: second,
                collection_expr: expr,
            },
            None => ForIntro {
                key_var: None,
                value_var: first,
                collection_expr: expr,
            },
        },
    )(input)
}

fn for_cond_expr(input: Input) -> IResult<Input, Formatted<Expression>> {
    preceded(tag("if"), decorated(ws, cut(expr), ws))(input)
}

fn parenthesis(input: Input) -> IResult<Input, Formatted<Expression>> {
    delimited(char('('), decorated(ws, cut(expr), ws), char_or_cut(')'))(input)
}

fn heredoc_start(input: Input) -> IResult<Input, (HeredocStripMode, (&str, Range<usize>))> {
    terminated(
        pair(
            alt((
                value(HeredocStripMode::Indent, tag("<<-")),
                value(HeredocStripMode::None, tag("<<")),
            )),
            with_span(cut(str_ident)),
        ),
        pair(space0, cut(line_ending)),
    )(input)
}

fn heredoc_end<'a>(delim: &'a str) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, ()> {
    value((), pair(space0, tag(delim)))
}

fn heredoc_content<'a>(
    strip: HeredocStripMode,
    delim: &'a str,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Template> {
    map_res(
        map_res(
            recognize(pair(
                many1_count(anychar_except(pair(line_ending, heredoc_end(delim)))),
                line_ending,
            )),
            |s| std::str::from_utf8(s.input()),
        ),
        move |input| {
            let content = match strip {
                HeredocStripMode::None => Cow::Borrowed(input),
                HeredocStripMode::Indent => dedent(input),
            };

            let input = Input::new(content.as_bytes());

            match heredoc_template(input) {
                Ok((_, template)) => Ok(template),
                Err(_) => Err(InternalError::new(
                    content,
                    ErrorKind::Context("HeredocTemplate"),
                )),
            }
        },
    )
}

fn heredoc(input: Input) -> IResult<Input, HeredocTemplate> {
    let (input, (strip, (delim, span))) = heredoc_start(input)?;

    let (input, template) = terminated(
        map(
            with_span(opt(heredoc_content(strip, delim))),
            |(template, span)| {
                let mut template = template.unwrap_or_default();
                template.set_span(span);
                template
            },
        ),
        cut(heredoc_end(delim)),
    )(input)?;

    Ok((
        input,
        HeredocTemplate {
            delimiter: Formatted::new_with_span(Identifier::unchecked(delim), span),
            template,
            strip,
        },
    ))
}

fn traversal_operator(input: Input) -> IResult<Input, TraversalOperator> {
    context(
        "TraversalOperator",
        alt((
            preceded(
                terminated(char('.'), ws),
                preceded(
                    // Must not match `for` object value grouping or func call expand final which
                    // are both `...`.
                    not(char('.')),
                    cut(alt((
                        value(TraversalOperator::AttrSplat, char('*')),
                        map(ident, TraversalOperator::GetAttr),
                        map(u64, TraversalOperator::LegacyIndex),
                    ))),
                ),
            ),
            delimited(
                terminated(char('['), ws),
                cut(alt((
                    value(TraversalOperator::FullSplat, char('*')),
                    map(expr, TraversalOperator::Index),
                ))),
                preceded(ws, char_or_cut(']')),
            ),
        )),
    )(input)
}

fn ident_or_func_call(input: Input) -> IResult<Input, Expression> {
    map(
        pair(with_span(str_ident), opt(preceded(ws, func_call))),
        |((ident, span), func_call)| match func_call {
            Some((args, expand_final)) => Expression::FuncCall(Box::new(FuncCall {
                name: Formatted::new_with_span(Identifier::unchecked(ident), span),
                args,
                expand_final,
            })),
            None => match ident {
                "null" => Expression::Null,
                "true" => Expression::Bool(true),
                "false" => Expression::Bool(false),
                var => Expression::Variable(Variable::unchecked(var)),
            },
        },
    )(input)
}

fn func_call(input: Input) -> IResult<Input, (Vec<Formatted<Expression>>, bool)> {
    delimited(
        char('('),
        alt((
            map(
                pair(
                    separated_list1(char(','), decorated(ws, expr, ws)),
                    opt(terminated(alt((tag(","), tag("..."))), ws)),
                ),
                |(args, trailer)| (args, trailer.as_deref() == Some(&&b"..."[..])),
            ),
            map(ws, |_| (Vec::new(), false)),
        )),
        char_or_cut(')'),
    )(input)
}

fn unary_operator(input: Input) -> IResult<Input, UnaryOperator> {
    alt((
        value(UnaryOperator::Neg, char('-')),
        value(UnaryOperator::Not, char('!')),
    ))(input)
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
        pair(spanned(unary_operator), prefix_decorated(sp, expr_term)),
        |(operator, expr)| {
            let op = UnaryOp { operator, expr };
            Expression::Operation(Box::new(Operation::Unary(op)))
        },
    )(input)
}

fn expr_term<'a>(input: Input<'a>) -> IResult<Input<'a>, Expression> {
    let (input, ch) = peek(anychar)(input)?;

    match ch {
        '"' => alt((
            map(string, Expression::String),
            map(quoted_string_template, Expression::Template),
        ))(input),
        '[' => array(input),
        '{' => object(input),
        '0'..='9' => map(number, Expression::Number)(input),
        '<' => map(heredoc, |heredoc| {
            Expression::HeredocTemplate(Box::new(heredoc))
        })(input),
        '-' => alt((
            map(preceded(pair(char('-'), sp), number), |n| {
                Expression::Number(-n)
            }),
            unary_op,
        ))(input),
        '!' => unary_op(input),
        '(' => map(parenthesis, |expr| Expression::Parenthesis(Box::new(expr)))(input),
        _ => alt((ident_or_func_call, fail))(input),
    }
}

pub fn expr_inner(input: Input) -> IResult<Input, Expression> {
    let traversal = with_span(many1(prefix_decorated(sp, traversal_operator)));

    let binary_op = with_span(pair(
        prefix_decorated(sp, binary_operator),
        prefix_decorated(sp, cut(expr)),
    ));

    let conditional = pair(
        preceded(pair(sp, char('?')), prefix_decorated(sp, cut(expr))),
        preceded(pair(sp, char_or_cut(':')), prefix_decorated(sp, cut(expr))),
    );

    map(
        tuple((
            with_span(expr_term),
            opt(traversal),
            opt(binary_op),
            opt(conditional),
        )),
        |((expr, span), traversal, binary_op, conditional)| {
            let start = span.start;
            let end = span.end;

            let (expr, end) = match traversal {
                Some((operators, span)) => {
                    let expr = Expression::Traversal(Box::new(Traversal {
                        expr: Formatted::new_with_span(expr, start..end),
                        operators,
                    }));

                    (expr, span.end)
                }
                None => (expr, end),
            };

            let (expr, end) = match binary_op {
                Some(((operator, rhs_expr), span)) => {
                    let expr = Expression::Operation(Box::new(Operation::Binary(BinaryOp {
                        lhs_expr: Formatted::new_with_span(expr, start..end),
                        operator,
                        rhs_expr,
                    })));

                    (expr, span.end)
                }
                None => (expr, end),
            };

            match conditional {
                Some((true_expr, false_expr)) => Expression::Conditional(Box::new(Conditional {
                    cond_expr: Formatted::new_with_span(expr, start..end),
                    true_expr,
                    false_expr,
                })),
                None => expr,
            }
        },
    )(input)
}

pub fn expr(input: Input) -> IResult<Input, Expression> {
    context("Expression", expr_inner)(input)
}
