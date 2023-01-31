use super::ast::{
    BinaryOp, Conditional, Expression, ForExpr, FuncCall, Heredoc, ObjectKey, Operation,
    TemplateExpr, Traversal, TraversalOperator, UnaryOp,
};
use super::{
    anything_except, char_or_cut, decorated,
    error::InternalError,
    ident, number, prefix_decorated, sp, str_ident, string, tag_or_cut,
    template::{heredoc_template, quoted_string_template},
    ws, ErrorKind, IResult,
};
use super::{spanned, with_span, Node, Span};
use crate::template;
use crate::Identifier;
use crate::{
    expr::{BinaryOperator, HeredocStripMode, Object, UnaryOperator, Variable},
    util::dedent,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, line_ending, one_of, space0, u64},
    combinator::{all_consuming, cut, fail, map, map_res, not, opt, peek, recognize, value},
    error::context,
    multi::{many1, many1_count, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};
use std::borrow::Cow;

fn array(input: Span) -> IResult<Span, Expression> {
    delimited(
        char('['),
        alt((
            map(for_list_expr, |expr| Expression::ForExpr(Box::new(expr))),
            map(array_items, Expression::Array),
        )),
        char_or_cut(']'),
    )(input)
}

fn array_items(input: Span) -> IResult<Span, Vec<Node<Expression>>> {
    alt((
        terminated(
            separated_list1(char(','), decorated(ws, expr, ws)),
            opt(terminated(char(','), ws)),
        ),
        map(ws, |_| Vec::new()),
    ))(input)
}

fn for_list_expr(input: Span) -> IResult<Span, ForExpr> {
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

fn object(input: Span) -> IResult<Span, Expression> {
    delimited(
        char('{'),
        alt((
            map(for_object_expr, |expr| Expression::ForExpr(Box::new(expr))),
            map(object_items, Expression::Object),
        )),
        char_or_cut('}'),
    )(input)
}

fn object_items(input: Span) -> IResult<Span, Object<Node<ObjectKey>, Node<Expression>>> {
    alt((
        map(
            many1(terminated(object_item, opt(pair(one_of(",\n"), ws)))),
            Object::from,
        ),
        map(ws, |_| Object::new()),
    ))(input)
}

fn object_item(input: Span) -> IResult<Span, (Node<ObjectKey>, Node<Expression>)> {
    separated_pair(
        decorated(
            ws,
            map(expr, |expr| {
                // Variable identifiers without traversal are treated as identifier object keys.
                //
                // Handle this case here by converting the variable into an identifier. This
                // avoids re-parsing the whole key-value pair when an identifier followed by a
                // traversal operator is encountered.
                if let Expression::Variable(variable) = expr {
                    ObjectKey::Identifier(variable.into_inner())
                } else {
                    ObjectKey::Expression(expr)
                }
            }),
            sp,
        ),
        cut(one_of("=:")),
        cut(decorated(sp, expr, ws)),
    )(input)
}

fn for_object_expr(input: Span) -> IResult<Span, ForExpr> {
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
    key_var: Option<Node<Identifier>>,
    value_var: Node<Identifier>,
    collection_expr: Node<Expression>,
}

fn for_intro(input: Span) -> IResult<Span, ForIntro> {
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

fn for_cond_expr(input: Span) -> IResult<Span, Node<Expression>> {
    preceded(tag("if"), decorated(ws, cut(expr), ws))(input)
}

fn parenthesis(input: Span) -> IResult<Span, Box<Node<Expression>>> {
    map(
        delimited(char('('), decorated(ws, cut(expr), ws), char_or_cut(')')),
        Box::new,
    )(input)
}

fn heredoc_start(input: Span) -> IResult<Span, (HeredocStripMode, Node<&str>)> {
    terminated(
        pair(
            alt((
                value(HeredocStripMode::Indent, tag("<<-")),
                value(HeredocStripMode::None, tag("<<")),
            )),
            spanned(cut(str_ident)),
        ),
        pair(space0, cut(line_ending)),
    )(input)
}

fn heredoc_end<'a>(delim: &'a str) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    recognize(tuple((line_ending, space0, tag(delim))))
}

fn heredoc_content_template<'a>(
    strip: HeredocStripMode,
    delim: &'a str,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, String> {
    let raw_content = map(
        terminated(
            recognize(many1_count(anything_except(heredoc_end(delim)))),
            heredoc_end(delim),
        ),
        |span: Span| *span,
    );

    map_res(raw_content, move |raw_content| {
        let content = match strip {
            HeredocStripMode::None => Cow::Borrowed(raw_content),
            HeredocStripMode::Indent => dedent(raw_content),
        };

        let input = Span::new(content.as_ref());
        let result = all_consuming(heredoc_template(heredoc_end(delim)))(input);

        result
            .map(|(_, template)| template)
            .map_err(|_| InternalError::new(raw_content, ErrorKind::Context("HeredocTemplate")))
    })
}

fn heredoc(input: Span) -> IResult<Span, Heredoc> {
    let (input, (strip, delim)) = heredoc_start(input)?;

    let nonempty_heredoc = heredoc_content_template(strip, delim.value());
    let empty_heredoc = terminated(space0, tag_or_cut(delim.value()));

    let (input, template) = spanned(alt((
        map(nonempty_heredoc, |mut content| {
            // Append the trailing newline here. This is easier than doing this via the parser combinators.
            content.push('\n');
            content
        }),
        map(empty_heredoc, |_| String::new()),
    )))(input)?;

    Ok((
        input,
        Heredoc {
            delimiter: delim.map_value(Identifier::unchecked),
            template,
            strip,
        },
    ))
}

fn traversal_operator(input: Span) -> IResult<Span, TraversalOperator> {
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

fn ident_or_func_call(input: Span) -> IResult<Span, Expression> {
    map(
        pair(with_span(str_ident), opt(preceded(ws, func_call))),
        |((ident, span), func_call)| match func_call {
            Some((args, expand_final)) => Expression::FuncCall(Box::new(FuncCall {
                name: Node::new(Identifier::unchecked(ident), span),
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

fn func_call(input: Span) -> IResult<Span, (Vec<Node<Expression>>, bool)> {
    delimited(
        char('('),
        alt((
            map(
                pair(
                    separated_list1(char(','), decorated(ws, expr, ws)),
                    opt(terminated(alt((tag(","), tag("..."))), ws)),
                ),
                |(args, trailer)| (args, trailer.as_deref() == Some(&"...")),
            ),
            map(ws, |_| (Vec::new(), false)),
        )),
        char_or_cut(')'),
    )(input)
}

fn unary_operator(input: Span) -> IResult<Span, UnaryOperator> {
    alt((
        value(UnaryOperator::Neg, char('-')),
        value(UnaryOperator::Not, char('!')),
    ))(input)
}

fn binary_operator(input: Span) -> IResult<Span, BinaryOperator> {
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

fn expr_term<'a>(input: Span<'a>) -> IResult<Span<'a>, Expression> {
    let (input, ch) = peek(anychar)(input)?;

    match ch {
        '"' => alt((
            map(string, Expression::String),
            map(quoted_string_template, |template| {
                let template = template::Template::from(template);
                let expr = TemplateExpr::QuotedString(template.to_string());
                Expression::TemplateExpr(Box::new(expr))
            }),
        ))(input),
        '[' => array(input),
        '{' => object(input),
        '0'..='9' => map(number, Expression::Number)(input),
        '<' => map(heredoc, |heredoc| {
            let expr = TemplateExpr::Heredoc(heredoc);
            Expression::TemplateExpr(Box::new(expr))
        })(input),
        '(' => map(parenthesis, Expression::Parenthesis)(input),
        _ => alt((ident_or_func_call, fail))(input),
    }
}

pub fn expr_inner(input: Span) -> IResult<Span, Expression> {
    let unary_op = with_span(terminated(unary_operator, ws));

    let traversal = with_span(many1(prefix_decorated(ws, traversal_operator)));

    let binary_op = with_span(pair(
        prefix_decorated(ws, binary_operator),
        prefix_decorated(ws, cut(expr)),
    ));

    let conditional = pair(
        preceded(pair(sp, char('?')), prefix_decorated(sp, cut(expr))),
        preceded(pair(sp, char_or_cut(':')), prefix_decorated(sp, cut(expr))),
    );

    map(
        tuple((
            opt(unary_op),
            with_span(expr_term),
            opt(traversal),
            opt(binary_op),
            opt(conditional),
        )),
        |(unary_op, (expr, span), traversal, binary_op, conditional)| {
            let start = span.start;
            let end = span.end;

            let (expr, start) = match unary_op {
                Some((operator, span)) => match (&operator, &expr) {
                    (UnaryOperator::Neg, Expression::Number(_)) => match (operator, expr) {
                        (UnaryOperator::Neg, Expression::Number(num)) => {
                            (Expression::Number(-num), span.start)
                        }
                        _ => unreachable!(),
                    },
                    _ => {
                        let expr = Expression::Operation(Box::new(Operation::Unary(UnaryOp {
                            operator: Node::new(operator, span.start..span.end),
                            expr: Node::new(expr, start..end),
                        })));

                        (expr, span.start)
                    }
                },
                None => (expr, start),
            };

            let (expr, end) = match traversal {
                Some((operators, span)) => {
                    let expr = Expression::Traversal(Box::new(Traversal {
                        expr: Node::new(expr, start..end),
                        operators,
                    }));

                    (expr, span.end)
                }
                None => (expr, end),
            };

            let (expr, end) = match binary_op {
                Some(((operator, rhs_expr), span)) => {
                    let expr = Expression::Operation(Box::new(Operation::Binary(BinaryOp {
                        lhs_expr: Node::new(expr, start..end),
                        operator,
                        rhs_expr,
                    })));

                    (expr, span.end)
                }
                None => (expr, end),
            };

            match conditional {
                Some((true_expr, false_expr)) => Expression::Conditional(Box::new(Conditional {
                    cond_expr: Node::new(expr, start..end),
                    true_expr,
                    false_expr,
                })),
                None => expr,
            }
        },
    )(input)
}

pub fn expr(input: Span) -> IResult<Span, Expression> {
    context("Expression", expr_inner)(input)
}
