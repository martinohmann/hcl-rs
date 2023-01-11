use super::{
    combinators::{opt_sep, sp_comment_delimited0, ws_comment_delimited0},
    comment::{sp_comment0, ws_comment0},
    primitives::{boolean, ident, null, number, str_ident, string},
};
use crate::{
    expr::{
        BinaryOp, BinaryOperator, Expression, ForExpr, FuncCall, Heredoc, HeredocStripMode, Object,
        ObjectKey, TemplateExpr, Traversal, TraversalOperator, UnaryOp, UnaryOperator, Variable,
    },
    util::is_templated,
    Conditional, Identifier,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, line_ending, one_of, space0, u64},
    combinator::{cut, map, not, opt, recognize},
    error::{context, ContextError, FromExternalError, ParseError},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::num::ParseIntError;

fn array<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Expression>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "array",
        map(
            delimited(
                terminated(char('['), ws_comment0),
                opt(terminated(
                    separated_list1(ws_comment_delimited0(char(',')), expr),
                    terminated(opt_sep(char(',')), ws_comment0),
                )),
                char(']'),
            ),
            Option::unwrap_or_default,
        ),
    )(input)
}

fn object<'a, E>(input: &'a str) -> IResult<&'a str, Object<ObjectKey, Expression>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "object",
        map(
            delimited(
                terminated(char('{'), ws_comment0),
                opt(many1(terminated(
                    object_key_value,
                    terminated(opt_sep(one_of(",\n")), ws_comment0),
                ))),
                char('}'),
            ),
            |items| Object::from(items.unwrap_or_default()),
        ),
    )(input)
}

fn object_key_value<'a, E>(input: &'a str) -> IResult<&'a str, (ObjectKey, Expression), E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    separated_pair(
        map(expr, |expr| {
            // Variable identifiers without traversal are treated as identifier object keys. This
            // allows us to avoid re-parsing the whole key-value pair when an identifier followed
            // by a traversal operator is encountered.
            if let Expression::Variable(variable) = expr {
                ObjectKey::Identifier(variable.into_inner())
            } else {
                ObjectKey::Expression(expr)
            }
        }),
        sp_comment_delimited0(one_of("=:")),
        cut(expr),
    )(input)
}

fn parenthesis<'a, E>(input: &'a str) -> IResult<&'a str, Box<Expression>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    map(
        delimited(tag("("), ws_comment_delimited0(expr), tag(")")),
        Box::new,
    )(input)
}

fn string_or_template<'a, E>(input: &'a str) -> IResult<&'a str, Expression, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    map(string, |s| {
        if is_templated(&s) {
            Expression::from(TemplateExpr::QuotedString(s))
        } else {
            Expression::String(s)
        }
    })(input)
}

fn heredoc_start<'a, E>(input: &'a str) -> IResult<&'a str, (HeredocStripMode, &'a str), E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    terminated(
        pair(
            alt((
                map(tag("<<-"), |_| HeredocStripMode::Indent),
                map(tag("<<"), |_| HeredocStripMode::None),
            )),
            str_ident,
        ),
        pair(space0, line_ending),
    )(input)
}

fn heredoc_end<'a, E>(delim: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    recognize(tuple((line_ending, space0, tag(delim))))
}

fn heredoc_template<'a, E>(delim: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        map(
            recognize(many1(preceded(not(heredoc_end(delim)), anychar))),
            |template| {
                // Append the trailing newline here. This is easier than doing this via the parser combinators.
                let mut template = template.to_owned();
                template.push('\n');
                template
            },
        )(input)
    }
}

fn heredoc<'a, E>(input: &'a str) -> IResult<&'a str, Heredoc, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    let (input, (strip, delim)) = heredoc_start(input)?;

    map(
        terminated(heredoc_template(delim), heredoc_end(delim)),
        move |template| Heredoc {
            delimiter: Identifier::unchecked(delim),
            template,
            strip,
        },
    )(input)
}

fn traversal_operator<'a, E>(input: &'a str) -> IResult<&'a str, TraversalOperator, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    alt((
        preceded(
            terminated(char('.'), ws_comment0),
            alt((
                map(char('*'), |_| TraversalOperator::AttrSplat),
                map(ident, TraversalOperator::GetAttr),
                map(u64, TraversalOperator::LegacyIndex),
            )),
        ),
        delimited(
            terminated(char('['), ws_comment0),
            alt((
                map(char('*'), |_| TraversalOperator::FullSplat),
                map(expr, TraversalOperator::Index),
            )),
            preceded(ws_comment0, char(']')),
        ),
    ))(input)
}

fn variable_or_func_call<'a, E>(input: &'a str) -> IResult<&'a str, Expression, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    map(
        pair(ident, opt(preceded(ws_comment0, func_sig))),
        |(name, sig)| match sig {
            Some((args, expand_final)) => Expression::from(FuncCall {
                name,
                args,
                expand_final,
            }),
            None => Expression::from(Variable::from(name)),
        },
    )(input)
}

fn func_sig<'a, E>(input: &'a str) -> IResult<&'a str, (Vec<Expression>, bool), E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "func signature",
        map(
            delimited(
                terminated(char('('), ws_comment0),
                opt(pair(
                    separated_list1(ws_comment_delimited0(char(',')), expr),
                    terminated(opt_sep(alt((tag(","), tag("...")))), ws_comment0),
                )),
                char(')'),
            ),
            |pair| {
                pair.map(|(args, trailer)| (args, trailer == Some("...")))
                    .unwrap_or_default()
            },
        ),
    )(input)
}

struct ForIntro {
    key_var: Option<Identifier>,
    value_var: Identifier,
    collection_expr: Expression,
}

fn for_intro<'a, E>(input: &'a str) -> IResult<&'a str, ForIntro, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    map(
        delimited(
            terminated(tag("for"), ws_comment0),
            tuple((
                ident,
                opt(preceded(ws_comment_delimited0(char(',')), ident)),
                preceded(ws_comment_delimited0(tag("in")), expr),
            )),
            preceded(ws_comment0, char(':')),
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

fn for_cond_expr<'a, E>(input: &'a str) -> IResult<&'a str, Expression, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    preceded(terminated(tag("if"), ws_comment0), expr)(input)
}

fn for_list_expr<'a, E>(input: &'a str) -> IResult<&'a str, ForExpr, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    map(
        tuple((
            terminated(for_intro, ws_comment0),
            expr,
            opt(preceded(ws_comment0, for_cond_expr)),
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

fn for_object_expr<'a, E>(input: &'a str) -> IResult<&'a str, ForExpr, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    map(
        tuple((
            terminated(for_intro, ws_comment0),
            separated_pair(expr, ws_comment_delimited0(tag("=>")), expr),
            opt(preceded(ws_comment0, tag("..."))),
            opt(preceded(ws_comment0, for_cond_expr)),
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

fn for_expr<'a, E>(input: &'a str) -> IResult<&'a str, ForExpr, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    alt((
        delimited(
            terminated(char('['), ws_comment0),
            for_list_expr,
            preceded(ws_comment0, char(']')),
        ),
        delimited(
            terminated(char('{'), ws_comment0),
            for_object_expr,
            preceded(ws_comment0, char('}')),
        ),
    ))(input)
}

fn unary_operator<'a, E>(input: &'a str) -> IResult<&'a str, UnaryOperator, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "unary operator",
        alt((
            map(char('-'), |_| UnaryOperator::Neg),
            map(char('!'), |_| UnaryOperator::Not),
        )),
    )(input)
}

fn binary_operator<'a, E>(input: &'a str) -> IResult<&'a str, BinaryOperator, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "binary operator",
        alt((
            map(tag("=="), |_| BinaryOperator::Eq),
            map(tag("!="), |_| BinaryOperator::NotEq),
            map(tag("<="), |_| BinaryOperator::LessEq),
            map(tag(">="), |_| BinaryOperator::GreaterEq),
            map(char('<'), |_| BinaryOperator::Less),
            map(char('>'), |_| BinaryOperator::Greater),
            map(char('+'), |_| BinaryOperator::Plus),
            map(char('-'), |_| BinaryOperator::Minus),
            map(char('*'), |_| BinaryOperator::Mul),
            map(char('/'), |_| BinaryOperator::Div),
            map(char('%'), |_| BinaryOperator::Mod),
            map(tag("&&"), |_| BinaryOperator::And),
            map(tag("||"), |_| BinaryOperator::Or),
        )),
    )(input)
}

fn expr_term<'a, E>(input: &'a str) -> IResult<&'a str, Expression, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    map(
        pair(
            alt((
                string_or_template,
                variable_or_func_call,
                map(number, Expression::Number),
                map(boolean, Expression::Bool),
                map(null, |_| Expression::Null),
                map(array, Expression::Array),
                map(object, Expression::Object),
                map(for_expr, Expression::from),
                map(parenthesis, Expression::Parenthesis),
                map(heredoc, Expression::from),
            )),
            many0(preceded(ws_comment0, traversal_operator)),
        ),
        |(expr, operators)| {
            if operators.is_empty() {
                expr
            } else {
                Expression::from(Traversal { expr, operators })
            }
        },
    )(input)
}

pub fn expr<'a, E>(input: &'a str) -> IResult<&'a str, Expression, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    let unary_op = terminated(unary_operator, ws_comment0);

    let binary_op = pair(ws_comment_delimited0(binary_operator), expr);

    let conditional = pair(
        sp_comment_delimited0(preceded(terminated(char('?'), sp_comment0), expr)),
        preceded(terminated(char(':'), sp_comment0), expr),
    );

    context(
        "expression",
        map(
            tuple((opt(unary_op), expr_term, opt(binary_op), opt(conditional))),
            |(unary_op, expr, binary_op, conditional)| {
                let lhs_expr = match unary_op {
                    Some(operator) => {
                        // Negative numbers are implemented as unary negation operations in the HCL
                        // spec. We'll convert these to negative numbers to make them more
                        // convenient to use.
                        match (operator, expr) {
                            (UnaryOperator::Neg, Expression::Number(num)) => {
                                Expression::Number(-num)
                            }
                            (operator, expr) => Expression::from(UnaryOp { operator, expr }),
                        }
                    }
                    None => expr,
                };

                let expr = match binary_op {
                    Some((operator, rhs_expr)) => Expression::from(BinaryOp {
                        lhs_expr,
                        operator,
                        rhs_expr,
                    }),
                    None => lhs_expr,
                };

                match conditional {
                    Some((true_expr, false_expr)) => Expression::from(Conditional {
                        cond_expr: expr,
                        true_expr,
                        false_expr,
                    }),
                    None => expr,
                }
            },
        ),
    )(input)
}
