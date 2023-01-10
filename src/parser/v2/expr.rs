use super::{
    combinators::{sp_comment_delimited0, ws_comment_delimited0},
    comment::ws_comment0,
    primitives::{boolean, ident, null, number, string},
};
use crate::{
    expr::{
        BinaryOp, BinaryOperator, Expression, ForExpr, FuncCall, Object, ObjectKey, TemplateExpr,
        Traversal, TraversalOperator, UnaryOp, UnaryOperator, Variable,
    },
    util::is_templated,
    Identifier,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of, u64},
    combinator::{cut, map, opt},
    error::{context, ContextError, FromExternalError, ParseError},
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::num::ParseIntError;

fn array<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Expression>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    let empty_array = map(delimited(char('['), ws_comment0, char(']')), |_| Vec::new());

    let non_empty_array = delimited(
        terminated(char('['), ws_comment0),
        separated_list1(ws_comment_delimited0(char(',')), expr),
        preceded(ws_comment_delimited0(opt(char(','))), char(']')),
    );

    context("array", alt((empty_array, non_empty_array)))(input)
}

fn object<'a, E>(input: &'a str) -> IResult<&'a str, Object<ObjectKey, Expression>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    let empty_object = map(delimited(char('{'), ws_comment0, char('}')), |_| {
        Object::new()
    });

    let non_empty_object = delimited(
        terminated(char('{'), ws_comment0),
        map(
            separated_list1(ws_comment_delimited0(opt(char(','))), object_key_value),
            Object::from_iter,
        ),
        preceded(ws_comment_delimited0(opt(char(','))), char('}')),
    );

    context("object", alt((empty_object, non_empty_object)))(input)
}

fn object_key_value<'a, E>(input: &'a str) -> IResult<&'a str, (ObjectKey, Expression), E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    let kv_sep = || sp_comment_delimited0(one_of("=:"));

    alt((
        separated_pair(map(ident, ObjectKey::Identifier), kv_sep(), cut(expr)),
        separated_pair(map(expr, ObjectKey::Expression), kv_sep(), cut(expr)),
    ))(input)
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
        separated_pair(ident, ws_comment0, opt(func_sig)),
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
    // Parse opening brace.
    let (input, _) = terminated(char('('), ws_comment0)(input)?;

    // Parse function arguments, if any.
    let (input, args) = separated_list0(ws_comment_delimited0(char(',')), expr)(input)?;

    let (input, expand_final) = if args.is_empty() {
        (input, false)
    } else {
        // Parse optional trailing comma or final argument expansion.
        let trailer = opt(alt((tag(","), tag("..."))));

        map(ws_comment_delimited0(trailer), |v| v == Some("..."))(input)?
    };

    // Eat closing brace.
    let (input, _) = char(')')(input)?;
    Ok((input, (args, expand_final)))
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

fn for_expr<'a, E>(input: &'a str) -> IResult<&'a str, ForExpr, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    alt((
        map(
            delimited(
                terminated(char('['), ws_comment0),
                tuple((
                    terminated(for_intro, ws_comment0),
                    expr,
                    opt(preceded(ws_comment0, for_cond_expr)),
                )),
                preceded(ws_comment0, char(']')),
            ),
            |(intro, value_expr, cond_expr)| ForExpr {
                key_var: intro.key_var,
                value_var: intro.value_var,
                collection_expr: intro.collection_expr,
                key_expr: None,
                value_expr,
                cond_expr,
                grouping: false,
            },
        ),
        map(
            delimited(
                terminated(char('{'), ws_comment0),
                tuple((
                    terminated(for_intro, ws_comment0),
                    separated_pair(expr, ws_comment_delimited0(tag("=>")), expr),
                    opt(preceded(ws_comment0, tag("..."))),
                    opt(preceded(ws_comment0, for_cond_expr)),
                )),
                preceded(ws_comment0, char('}')),
            ),
            |(intro, (key_expr, value_expr), grouping, cond_expr)| ForExpr {
                key_var: intro.key_var,
                value_var: intro.value_var,
                collection_expr: intro.collection_expr,
                key_expr: Some(key_expr),
                value_expr,
                cond_expr,
                grouping: grouping.is_some(),
            },
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
    context(
        "expression",
        map(
            tuple((
                opt(terminated(unary_operator, ws_comment0)),
                expr_term,
                opt(pair(ws_comment_delimited0(binary_operator), expr)),
            )),
            |(operator, expr, binary_op)| {
                let lhs_expr = match operator {
                    Some(operator) => match (operator, expr) {
                        (UnaryOperator::Neg, Expression::Number(num)) => Expression::Number(-num),
                        (operator, expr) => Expression::from(UnaryOp { operator, expr }),
                    },
                    None => expr,
                };

                match binary_op {
                    Some((operator, rhs_expr)) => Expression::from(BinaryOp {
                        lhs_expr,
                        operator,
                        rhs_expr,
                    }),
                    None => lhs_expr,
                }
            },
        ),
    )(input)
}
