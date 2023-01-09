use super::{
    combinators::{sp_comment_delimited0, ws_comment_delimited0},
    comment::ws_comment0,
    ident,
    string::string,
};
use crate::{
    expr::{
        Expression, ForExpr, FuncCall, Object, ObjectKey, TemplateExpr, Traversal,
        TraversalOperator, Variable,
    },
    util::is_templated,
    Number,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, i64 as i64_num, one_of, u64 as u64_num},
    combinator::{map, map_opt, opt, value},
    error::{context, ContextError, FromExternalError, ParseError},
    multi::{many0, separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};
use std::num::ParseIntError;

fn number<'a, E>(input: &'a str) -> IResult<&'a str, Number, E>
where
    E: ParseError<&'a str>,
{
    alt((
        map_opt(double, Number::from_f64),
        map(u64_num, Number::from),
        map(i64_num, Number::from),
    ))(input)
}

fn boolean<'a, E>(input: &'a str) -> IResult<&'a str, bool, E>
where
    E: ParseError<&'a str>,
{
    let true_tag = value(true, tag("true"));
    let false_tag = value(false, tag("false"));

    alt((true_tag, false_tag))(input)
}

fn null<'a, E>(input: &'a str) -> IResult<&'a str, (), E>
where
    E: ParseError<&'a str>,
{
    value((), tag("null"))(input)
}

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
            separated_list1(ws_comment_delimited0(char(',')), object_key_value),
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
    separated_pair(object_key, sp_comment_delimited0(one_of("=:")), expr)(input)
}

fn object_key<'a, E>(input: &'a str) -> IResult<&'a str, ObjectKey, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    alt((
        map(ident, ObjectKey::Identifier),
        map(expr, ObjectKey::Expression),
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
                map(u64_num, TraversalOperator::LegacyIndex),
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
        separated_pair(ident, ws_comment0, opt(func_args)),
        |(name, args)| match args {
            Some((args, expand_final)) => Expression::from(FuncCall {
                name,
                args,
                expand_final,
            }),
            None => Expression::from(Variable::from(name)),
        },
    )(input)
}

fn func_args<'a, E>(input: &'a str) -> IResult<&'a str, (Vec<Expression>, bool), E>
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

fn for_expr<'a, E>(input: &'a str) -> IResult<&'a str, Box<ForExpr>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    unimplemented!()
}

fn expr_term<'a, E>(input: &'a str) -> IResult<&'a str, Expression, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    map(
        pair(
            alt((
                string_or_template,
                map(number, Expression::Number),
                map(boolean, Expression::Bool),
                map(null, |_| Expression::Null),
                map(array, Expression::Array),
                map(object, Expression::Object),
                variable_or_func_call,
                map(for_expr, Expression::ForExpr),
                map(parenthesis, Expression::Parenthesis),
            )),
            many0(ws_comment_delimited0(traversal_operator)),
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
    context("expression", expr_term)(input)
}
