use super::Span;
use super::{
    char_or_cut, expr::expr, ident, literal, string_fragment, string_literal, tag_or_cut,
    ws_delimited, ws_preceded, IResult, StringFragment,
};
use crate::expr::Expression;
use crate::template::{
    Directive, Element, ForDirective, IfDirective, Interpolation, StripMode, Template,
};
use crate::Identifier;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    multi::{fold_many1, many0},
    sequence::{delimited, pair, preceded, terminated, tuple},
};

fn build_literal<'a, F>(literal: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, String>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, Span<'a>>,
{
    fold_many1(
        string_fragment(literal),
        String::new,
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
            }
            string
        },
    )
}

fn interpolation(input: Span) -> IResult<Span, Interpolation> {
    map(template_tag("${", cut(expr)), |(expr, strip)| {
        Interpolation { expr, strip }
    })(input)
}

fn template_tag<'a, F, O>(
    start_tag: &'a str,
    inner: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, (O, StripMode)>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    map(
        tuple((
            preceded(tag(start_tag), opt(char('~'))),
            ws_delimited(inner),
            terminated(opt(char('~')), char_or_cut('}')),
        )),
        |(strip_start, output, strip_end)| {
            (output, (strip_start.is_some(), strip_end.is_some()).into())
        },
    )
}

fn if_directive(input: Span) -> IResult<Span, IfDirective> {
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

    let endif_expr = map(template_tag("%{", tag_or_cut("endif")), |(_, strip)| strip);

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
    )(input)
}

fn for_directive(input: Span) -> IResult<Span, ForDirective> {
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
                    preceded(tag_or_cut("in"), ws_preceded(cut(expr))),
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

    let endfor_expr = map(template_tag("%{", tag_or_cut("endfor")), |(_, strip)| strip);

    map(pair(for_expr, endfor_expr), |(for_expr, endfor_strip)| {
        ForDirective {
            key_var: for_expr.key_var,
            value_var: for_expr.value_var,
            collection_expr: for_expr.collection_expr,
            template: for_expr.template,
            for_strip: for_expr.strip,
            endfor_strip,
        }
    })(input)
}

fn directive(input: Span) -> IResult<Span, Directive> {
    alt((
        map(if_directive, Directive::If),
        map(for_directive, Directive::For),
    ))(input)
}

fn build_template<'a, F>(literal: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Template>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, String>,
{
    map(
        many0(alt((
            map(literal, Element::Literal),
            map(interpolation, Element::Interpolation),
            map(directive, Element::Directive),
        ))),
        Template::from_iter,
    )
}

pub fn quoted_string_template(input: Span) -> IResult<Span, Template> {
    delimited(
        char('"'),
        build_template(build_literal(string_literal)),
        char('"'),
    )(input)
}

pub fn heredoc_template<'a, F>(
    heredoc_end: F,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Template>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, Span<'a>>,
{
    build_template(map(
        literal(alt((tag("${"), tag("%{"), heredoc_end))),
        |s: Span| s.fragment().to_string(),
    ))
}

pub fn template(input: Span) -> IResult<Span, Template> {
    build_template(build_literal(literal(alt((
        tag("\\"),
        tag("${"),
        tag("%{"),
    )))))(input)
}
