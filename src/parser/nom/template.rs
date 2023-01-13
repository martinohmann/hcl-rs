use super::{expr::expr, ident, ws_delimited, ws_preceded};
use crate::expr::Expression;
use crate::template::{
    Directive, Element, ForDirective, IfDirective, Interpolation, StripMode, Template,
};
use crate::Identifier;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char},
    combinator::{cut, map, not, opt, recognize},
    error::context,
    multi::{many0, many1},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

fn literal(input: &str) -> IResult<&str, String> {
    context(
        "literal",
        map(
            recognize(many1(alt((
                tag("$${"),
                tag("%%{"),
                recognize(preceded(not(alt((tag("${"), tag("%{")))), anychar)),
            )))),
            ToOwned::to_owned,
        ),
    )(input)
}

fn interpolation(input: &str) -> IResult<&str, Interpolation> {
    context(
        "interpolation",
        map(template_tag("${", expr), |(expr, strip)| Interpolation {
            expr,
            strip,
        }),
    )(input)
}

fn template_tag<'a, F, O>(
    start_tag: &'a str,
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, (O, StripMode)>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    map(
        tuple((
            preceded(tag(start_tag), opt(char('~'))),
            ws_delimited(inner),
            terminated(opt(char('~')), cut(char('}'))),
        )),
        |(strip_start, output, strip_end)| {
            (output, (strip_start.is_some(), strip_end.is_some()).into())
        },
    )
}

fn if_directive(input: &str) -> IResult<&str, IfDirective> {
    struct IfExpr {
        cond_expr: Expression,
        template: Template,
        strip: StripMode,
    }

    #[derive(Default)]
    struct ElseExpr {
        template: Option<Template>,
        strip: StripMode,
    }

    let if_expr = map(
        pair(
            template_tag("%{", preceded(tag("if"), ws_preceded(cut(expr)))),
            template,
        ),
        |((cond_expr, strip), template)| IfExpr {
            cond_expr,
            template,
            strip,
        },
    );

    let else_expr = map(
        pair(template_tag("%{", tag("else")), template),
        |((_, strip), template)| ElseExpr {
            template: Some(template),
            strip,
        },
    );

    let endif_expr = map(template_tag("%{", tag("endif")), |(_, strip)| strip);

    context(
        "if directive",
        map(
            tuple((if_expr, opt(else_expr), endif_expr)),
            |(if_expr, else_expr, endif_strip)| {
                let else_expr = else_expr.unwrap_or_default();

                IfDirective {
                    cond_expr: if_expr.cond_expr,
                    true_template: if_expr.template,
                    false_template: else_expr.template,
                    if_strip: if_expr.strip,
                    else_strip: else_expr.strip,
                    endif_strip,
                }
            },
        ),
    )(input)
}

fn for_directive(input: &str) -> IResult<&str, ForDirective> {
    struct ForExpr {
        key_var: Option<Identifier>,
        value_var: Identifier,
        collection_expr: Expression,
        template: Template,
        strip: StripMode,
    }

    let for_expr = map(
        pair(
            template_tag(
                "%{",
                tuple((
                    preceded(tag("for"), ws_delimited(cut(ident))),
                    opt(preceded(char(','), ws_delimited(cut(ident)))),
                    preceded(tag("in"), ws_preceded(cut(expr))),
                )),
            ),
            template,
        ),
        |(((key_var, value_var, collection_expr), strip), template)| {
            let (key_var, value_var) = match value_var {
                Some(value_var) => (Some(key_var), value_var),
                None => (None, key_var),
            };
            ForExpr {
                key_var,
                value_var,
                collection_expr,
                template,
                strip,
            }
        },
    );

    let endfor_expr = map(template_tag("%{", tag("endfor")), |(_, strip)| strip);

    context(
        "if directive",
        map(pair(for_expr, endfor_expr), |(for_expr, endfor_strip)| {
            ForDirective {
                key_var: for_expr.key_var,
                value_var: for_expr.value_var,
                collection_expr: for_expr.collection_expr,
                template: for_expr.template,
                for_strip: for_expr.strip,
                endfor_strip,
            }
        }),
    )(input)
}

fn directive(input: &str) -> IResult<&str, Directive> {
    context(
        "directive",
        alt((
            map(if_directive, Directive::If),
            map(for_directive, Directive::For),
        )),
    )(input)
}

pub fn template(input: &str) -> IResult<&str, Template> {
    context(
        "template",
        map(
            many0(alt((
                map(literal, Element::Literal),
                map(interpolation, Element::Interpolation),
                map(directive, Element::Directive),
            ))),
            Template::from_iter,
        ),
    )(input)
}
