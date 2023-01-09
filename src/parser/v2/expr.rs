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
    Identifier, Number,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of, u64 as u64_num},
    combinator::{cut, map, map_opt, opt, recognize, value},
    error::{context, ContextError, FromExternalError, ParseError},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::num::ParseIntError;

fn decimal<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    recognize(many1(one_of("0123456789")))(input)
}

fn float<'a, E>(input: &'a str) -> IResult<&'a str, f64, E>
where
    E: ParseError<&'a str>,
{
    map_opt(
        alt((
            // Case one: .42
            recognize(tuple((
                char('.'),
                decimal,
                opt(tuple((one_of("eE"), opt(one_of("+-")), decimal))),
            ))),
            // Case two: 42e42 and 42.42e42
            recognize(tuple((
                decimal,
                opt(preceded(char('.'), decimal)),
                one_of("eE"),
                opt(one_of("+-")),
                decimal,
            ))),
            // Case three: 42. and 42.42
            recognize(tuple((decimal, char('.'), opt(decimal)))),
            // Integer
        )),
        |v| v.parse().ok(),
    )(input)
}

fn integer<'a, E>(input: &'a str) -> IResult<&'a str, u64, E>
where
    E: ParseError<&'a str>,
{
    map_opt(decimal, |v| v.parse().ok())(input)
}

fn number<'a, E>(input: &'a str) -> IResult<&'a str, Number, E>
where
    E: ParseError<&'a str>,
{
    alt((map_opt(float, Number::from_f64), map(integer, Number::from)))(input)
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
        pair(
            preceded(
                tag("for"),
                ws_comment_delimited0(separated_pair(
                    ident,
                    ws_comment_delimited0(char(',')),
                    opt(ident),
                )),
            ),
            terminated(preceded(tag("in"), ws_comment_delimited0(expr)), char(':')),
        ),
        |((first, second), expr)| match second {
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
                char('['),
                ws_comment_delimited0(separated_pair(
                    separated_pair(for_intro, ws_comment0, expr),
                    ws_comment0,
                    opt(for_cond_expr),
                )),
                char(']'),
            ),
            |((intro, value_expr), cond_expr)| ForExpr {
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
                char('{'),
                ws_comment_delimited0(separated_pair(
                    separated_pair(
                        for_intro,
                        ws_comment0,
                        separated_pair(expr, ws_comment_delimited0(tag("=>")), expr),
                    ),
                    ws_comment0,
                    separated_pair(opt(tag("...")), ws_comment0, opt(for_cond_expr)),
                )),
                char('}'),
            ),
            |((intro, (key_expr, value_expr)), (grouping, cond_expr))| ForExpr {
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
