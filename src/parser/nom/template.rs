use std::num::ParseIntError;

use super::primitives::ident;
use crate::{
    template::{Directive, Element, ForDirective, IfDirective, Interpolation, StripMode, Template},
    Expression, Identifier,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char},
    combinator::{map, not, opt, recognize, value},
    error::{context, ContextError, FromExternalError, ParseError},
    multi::{many0, many1},
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

use super::{combinators::ws_comment_delimited0, comment::ws_comment0, expr::expr};

fn literal<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "literal",
        recognize(many1(alt((
            tag("$${"),
            tag("%%{"),
            recognize(preceded(not(alt((tag("${"), tag("%{")))), anychar)),
        )))),
    )(input)
}

fn interpolation<'a, E>(input: &'a str) -> IResult<&'a str, Interpolation, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "interpolation",
        map(
            tuple((
                interpolation_start,
                ws_comment_delimited0(expr),
                element_end,
            )),
            |(strip_start, expr, strip_end)| Interpolation {
                expr,
                strip: StripMode::from((strip_start, strip_end)),
            },
        ),
    )(input)
}

fn interpolation_start<'a, E>(input: &'a str) -> IResult<&'a str, bool, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    alt((value(true, tag("${~")), value(false, tag("${"))))(input)
}

fn directive_start<'a, E>(input: &'a str) -> IResult<&'a str, bool, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    alt((value(true, tag("%{~")), value(false, tag("%{"))))(input)
}

fn element_end<'a, E>(input: &'a str) -> IResult<&'a str, bool, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    alt((value(true, tag("~}")), value(false, tag("}"))))(input)
}

fn if_directive<'a, E>(input: &'a str) -> IResult<&'a str, IfDirective, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    struct IfExpr {
        cond_expr: Expression,
        template: Template,
        strip: StripMode,
    }

    let if_expr = map(
        tuple((
            terminated(directive_start, preceded(ws_comment0, tag("if"))),
            ws_comment_delimited0(expr),
            element_end,
            template,
        )),
        |(strip_start, cond_expr, strip_end, true_template)| IfExpr {
            cond_expr,
            template: true_template,
            strip: StripMode::from((strip_start, strip_end)),
        },
    );

    struct ElseExpr {
        template: Template,
        strip: StripMode,
    }

    let else_expr = map(
        tuple((
            terminated(directive_start, ws_comment_delimited0(tag("else"))),
            element_end,
            template,
        )),
        |(strip_start, strip_end, false_template)| ElseExpr {
            template: false_template,
            strip: StripMode::from((strip_start, strip_end)),
        },
    );

    struct EndIfExpr {
        strip: StripMode,
    }

    let endif_expr = map(
        separated_pair(
            directive_start,
            ws_comment_delimited0(tag("endif")),
            element_end,
        ),
        |(strip_start, strip_end)| EndIfExpr {
            strip: StripMode::from((strip_start, strip_end)),
        },
    );

    context(
        "if directive",
        map(
            tuple((if_expr, opt(else_expr), endif_expr)),
            |(if_expr, else_expr, endif_expr)| {
                let (false_template, else_strip) = match else_expr {
                    Some(else_expr) => (Some(else_expr.template), else_expr.strip),
                    None => (None, StripMode::default()),
                };

                IfDirective {
                    cond_expr: if_expr.cond_expr,
                    true_template: if_expr.template,
                    false_template,
                    if_strip: if_expr.strip,
                    else_strip,
                    endif_strip: endif_expr.strip,
                }
            },
        ),
    )(input)
}

fn for_directive<'a, E>(input: &'a str) -> IResult<&'a str, ForDirective, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    struct ForExpr {
        key_var: Option<Identifier>,
        value_var: Identifier,
        collection_expr: Expression,
        template: Template,
        strip: StripMode,
    }

    let for_expr = map(
        tuple((
            terminated(directive_start, preceded(ws_comment0, tag("for"))),
            ws_comment_delimited0(ident),
            opt(preceded(char(','), ws_comment_delimited0(ident))),
            preceded(tag("in"), ws_comment_delimited0(expr)),
            element_end,
            template,
        )),
        |(strip_start, key_var, value_var, collection_expr, strip_end, template)| {
            let (key_var, value_var) = match value_var {
                Some(value_var) => (Some(key_var), value_var),
                None => (None, key_var),
            };
            ForExpr {
                key_var,
                value_var,
                collection_expr,
                template,
                strip: StripMode::from((strip_start, strip_end)),
            }
        },
    );

    struct EndForExpr {
        strip: StripMode,
    }

    let endfor_expr = map(
        separated_pair(
            directive_start,
            ws_comment_delimited0(tag("endfor")),
            element_end,
        ),
        |(strip_start, strip_end)| EndForExpr {
            strip: StripMode::from((strip_start, strip_end)),
        },
    );

    context(
        "if directive",
        map(pair(for_expr, endfor_expr), |(for_expr, endfor_expr)| {
            ForDirective {
                key_var: for_expr.key_var,
                value_var: for_expr.value_var,
                collection_expr: for_expr.collection_expr,
                template: for_expr.template,
                for_strip: for_expr.strip,
                endfor_strip: endfor_expr.strip,
            }
        }),
    )(input)
}

fn directive<'a, E>(input: &'a str) -> IResult<&'a str, Directive, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "directive",
        alt((
            map(if_directive, Directive::If),
            map(for_directive, Directive::For),
        )),
    )(input)
}

pub fn template<'a, E>(input: &'a str) -> IResult<&'a str, Template, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "template",
        map(
            many0(alt((
                map(literal, |literal| Element::Literal(literal.to_owned())),
                map(interpolation, Element::Interpolation),
                map(directive, Element::Directive),
            ))),
            Template::from_iter,
        ),
    )(input)
}
