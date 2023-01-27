use super::ast::{
    BinaryOp, Conditional, Expression, ForExpr, FuncCall, Heredoc, ObjectKey, Operation, Template,
    TemplateExpr, Traversal, TraversalOperator, UnaryOp,
};
use super::{
    anything_except, char_or_cut,
    error::InternalError,
    ident, number, opt_sep, sp_delimited, str_ident, string, tag_or_cut,
    template::{heredoc_template, quoted_string_template},
    ws_delimited, ws_preceded, ws_terminated, ErrorKind, IResult,
};
use super::{spanned, Span, Spanned};
use crate::template;
use crate::Identifier;
use crate::{
    expr::{BinaryOperator, HeredocStripMode, Object, UnaryOperator, Variable},
    util::dedent,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, one_of, space0, u64},
    combinator::{all_consuming, cut, fail, map, map_res, not, opt, recognize, value},
    error::context,
    multi::{many0, many1, many1_count, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};
use std::borrow::Cow;

fn array(input: Span) -> IResult<Span, Expression> {
    delimited(
        ws_terminated(char('[')),
        alt((
            map(for_list_expr, |expr| Expression::ForExpr(Box::new(expr))),
            map(array_items, Expression::Array),
        )),
        ws_preceded(char_or_cut(']')),
    )(input)
}

fn array_items(input: Span) -> IResult<Span, Vec<Spanned<Expression>>> {
    map(
        opt(terminated(
            separated_list1(ws_delimited(char(',')), expr),
            opt_sep(char(',')),
        )),
        Option::unwrap_or_default,
    )(input)
}

fn for_list_expr(input: Span) -> IResult<Span, ForExpr> {
    map(
        tuple((
            ws_terminated(for_intro),
            cut(expr),
            opt(ws_preceded(for_cond_expr)),
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
        ws_terminated(char('{')),
        alt((
            map(ws_terminated(for_object_expr), |expr| {
                Expression::ForExpr(Box::new(expr))
            }),
            map(object_items, Expression::Object),
        )),
        char_or_cut('}'),
    )(input)
}

fn object_items(input: Span) -> IResult<Span, Object<Spanned<ObjectKey>, Spanned<Expression>>> {
    map(
        opt(many1(terminated(
            object_item,
            ws_terminated(opt_sep(one_of(",\n"))),
        ))),
        |items| Object::from(items.unwrap_or_default()),
    )(input)
}

fn object_item(input: Span) -> IResult<Span, (Spanned<ObjectKey>, Spanned<Expression>)> {
    separated_pair(
        map(expr, |spanned| {
            // Variable identifiers without traversal are treated as identifier object keys.
            //
            // Handle this case here by converting the variable into an identifier. This
            // avoids re-parsing the whole key-value pair when an identifier followed by a
            // traversal operator is encountered.
            spanned.map_value(|expr| {
                if let Expression::Variable(variable) = expr {
                    ObjectKey::Identifier(variable.into_inner())
                } else {
                    ObjectKey::Expression(expr)
                }
            })
        }),
        sp_delimited(cut(one_of("=:"))),
        cut(expr),
    )(input)
}

fn for_object_expr(input: Span) -> IResult<Span, ForExpr> {
    map(
        tuple((
            ws_terminated(for_intro),
            separated_pair(cut(expr), ws_delimited(tag_or_cut("=>")), cut(expr)),
            opt(ws_preceded(tag("..."))),
            opt(ws_preceded(for_cond_expr)),
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

struct ForIntro<'a> {
    key_var: Option<Spanned<'a, Identifier>>,
    value_var: Spanned<'a, Identifier>,
    collection_expr: Spanned<'a, Expression<'a>>,
}

fn for_intro(input: Span) -> IResult<Span, ForIntro> {
    map(
        delimited(
            ws_terminated(tag("for")),
            tuple((
                cut(spanned(ident)),
                opt(preceded(ws_delimited(char(',')), cut(spanned(ident)))),
                preceded(ws_delimited(tag_or_cut("in")), cut(expr)),
            )),
            ws_preceded(char_or_cut(':')),
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

fn for_cond_expr(input: Span) -> IResult<Span, Spanned<Expression>> {
    preceded(ws_terminated(tag("if")), cut(expr))(input)
}

fn parenthesis(input: Span) -> IResult<Span, Box<Spanned<Expression>>> {
    map(
        delimited(char('('), ws_delimited(cut(expr)), char_or_cut(')')),
        Box::new,
    )(input)
}

fn heredoc_start(input: Span) -> IResult<Span, (HeredocStripMode, Span)> {
    terminated(
        pair(
            alt((
                value(HeredocStripMode::Indent, tag("<<-")),
                value(HeredocStripMode::None, tag("<<")),
            )),
            cut(str_ident),
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
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Spanned<'a, Template<'a>>> {
    let raw_content = terminated(
        recognize(many1_count(anything_except(heredoc_end(delim)))),
        heredoc_end(delim),
    );

    map_res(raw_content, move |raw_content| {
        // let content = match strip {
        //     HeredocStripMode::None => Cow::Borrowed(*raw_content),
        //     HeredocStripMode::Indent => dedent(*raw_content),
        // };

        // let input = Span::new(&content);
        // let result = all_consuming(heredoc_template(heredoc_end(delim)))(input);
        let _ = strip;
        let result = all_consuming(heredoc_template(heredoc_end(delim)))(raw_content);

        result
            .map(|(_, template)| template)
            .map_err(|_| InternalError::new(raw_content, ErrorKind::Context("HeredocTemplate")))
    })
}

fn heredoc(input: Span) -> IResult<Span, Heredoc> {
    let (input, (strip, delim)) = heredoc_start(input)?;

    let nonempty_heredoc = heredoc_content_template(strip, *delim);
    let empty_heredoc = terminated(space0, tag_or_cut(*delim));

    map(
        alt((
            map(nonempty_heredoc, |spanned| {
                // Append the trailing newline here. This is easier than doing this via the parser combinators.
                let mut content = template::Template::from(spanned.value).to_string();
                content.push('\n');
                content
            }),
            map(empty_heredoc, |_| String::new()),
        )),
        move |template| Heredoc {
            delimiter: Identifier::unchecked(*delim),
            template,
            strip,
        },
    )(input)
}

fn template_expr(input: Span) -> IResult<Span, TemplateExpr> {
    alt((
        map(quoted_string_template, |spanned| {
            TemplateExpr::QuotedString(template::Template::from(spanned.value).to_string())
        }),
        map(heredoc, TemplateExpr::Heredoc),
    ))(input)
}

fn traversal_operator(input: Span) -> IResult<Span, TraversalOperator> {
    context(
        "TraversalOperator",
        alt((
            preceded(
                ws_terminated(char('.')),
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
                ws_terminated(char('[')),
                cut(alt((
                    value(TraversalOperator::FullSplat, char('*')),
                    map(expr, |expr| TraversalOperator::Index(expr.value)),
                ))),
                ws_preceded(char_or_cut(']')),
            ),
        )),
    )(input)
}

fn ident_or_func_call(input: Span) -> IResult<Span, Expression> {
    map(
        pair(spanned(str_ident), opt(ws_preceded(func_call))),
        |(ident, func_call)| match func_call {
            Some((args, expand_final)) => Expression::FuncCall(Box::new(FuncCall {
                name: ident.map_value(|value| Identifier::unchecked(*value)),
                args,
                expand_final,
            })),
            None => match *ident.value {
                "null" => Expression::Null,
                "true" => Expression::Bool(true),
                "false" => Expression::Bool(false),
                var => Expression::Variable(Variable::unchecked(var)),
            },
        },
    )(input)
}

fn func_call(input: Span) -> IResult<Span, (Vec<Spanned<Expression>>, bool)> {
    map(
        delimited(
            ws_terminated(char('(')),
            opt(pair(
                separated_list1(ws_delimited(char(',')), expr),
                ws_terminated(opt_sep(alt((tag(","), tag("..."))))),
            )),
            char_or_cut(')'),
        ),
        |pair| {
            pair.map(|(args, trailer)| (args, trailer.as_deref() == Some(&"...")))
                .unwrap_or_default()
        },
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

fn expr_term(input: Span) -> IResult<Span, Expression> {
    alt((
        map(number, Expression::Number),
        map(string, Expression::String),
        ident_or_func_call,
        array,
        object,
        map(template_expr, |expr| {
            Expression::TemplateExpr(Box::new(expr))
        }),
        map(parenthesis, |expr| Expression::Parenthesis(expr)),
        fail,
    ))(input)
}

pub fn expr_inner(input: Span) -> IResult<Span, Expression> {
    let unary_op = ws_terminated(unary_operator);

    let traversal = many1(ws_preceded(spanned(traversal_operator)));

    let binary_op = pair(ws_delimited(binary_operator), cut(expr));

    let conditional = pair(
        preceded(sp_delimited(char('?')), cut(expr)),
        preceded(sp_delimited(char_or_cut(':')), cut(expr)),
    );

    map(
        tuple((
            opt(unary_op),
            expr_term,
            opt(traversal),
            opt(binary_op),
            opt(conditional),
        )),
        |(unary_op, expr, traversal, binary_op, conditional)| {
            let expr = if let Some(operator) = unary_op {
                // Negative numbers are implemented as unary negation operations in the HCL
                // spec. We'll convert these to negative numbers to make them more
                // convenient to use.
                match (operator, expr) {
                    (UnaryOperator::Neg, Expression::Number(num)) => Expression::Number(-num),
                    (operator, expr) => {
                        Expression::Operation(Box::new(Operation::Unary(UnaryOp {
                            operator,
                            expr,
                        })))
                    }
                }
            } else {
                expr
            };

            let expr = match traversal {
                Some(operators) => Expression::Traversal(Box::new(Traversal { expr, operators })),
                None => expr,
            };

            let expr = if let Some((operator, rhs_expr)) = binary_op {
                Expression::Operation(Box::new(Operation::Binary(BinaryOp {
                    lhs_expr: expr,
                    operator,
                    rhs_expr,
                })))
            } else {
                expr
            };

            if let Some((true_expr, false_expr)) = conditional {
                Expression::Conditional(Box::new(Conditional {
                    cond_expr: expr,
                    true_expr,
                    false_expr,
                }))
            } else {
                expr
            }
        },
    )(input)
}

pub fn expr(input: Span) -> IResult<Span, Spanned<Expression>> {
    context("Expression", spanned(expr_inner))(input)
}
