use super::ast::{
    Directive, Element, ElseTemplateExpr, EndforTemplateExpr, EndifTemplateExpr, ForDirective,
    ForTemplateExpr, IfDirective, IfTemplateExpr, Interpolation, Template,
};
use super::{
    build_string, char_or_cut, decor, expr::expr, ident, literal, span, spanned, string_fragment,
    string_literal, tag_or_cut, ws, IResult, Input,
};
use crate::template::StripMode;
use crate::InternalString;
use nom::sequence::separated_pair;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated, tuple},
};

fn interpolation(input: Input) -> IResult<Input, Interpolation> {
    map(
        template_tag("${", decor(ws, cut(expr), ws)),
        |(expr, strip)| Interpolation::new(expr, strip),
    )(input)
}

fn template_tag<'a, F, O>(
    start_tag: &'a str,
    inner: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, (O, StripMode)>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
{
    map(
        tuple((
            preceded(tag(start_tag), opt(char('~'))),
            inner,
            terminated(opt(char('~')), char_or_cut('}')),
        )),
        |(strip_start, output, strip_end)| {
            (output, (strip_start.is_some(), strip_end.is_some()).into())
        },
    )
}

fn if_directive(input: Input) -> IResult<Input, IfDirective> {
    let if_expr = map(
        pair(
            template_tag(
                "%{",
                pair(terminated(span(ws), tag("if")), decor(ws, cut(expr), ws)),
            ),
            spanned(template),
        ),
        |(((preamble, cond_expr), strip), template)| {
            let mut expr = IfTemplateExpr::new(cond_expr, template, strip);
            expr.set_preamble(preamble);
            expr
        },
    );

    let else_expr = map(
        pair(
            template_tag("%{", separated_pair(span(ws), tag("else"), span(ws))),
            spanned(template),
        ),
        |(((preamble, trailing), strip), template)| {
            let mut expr = ElseTemplateExpr::new(template, strip);
            expr.set_preamble(preamble);
            expr.set_trailing(trailing);
            expr
        },
    );

    let endif_expr = map(
        template_tag(
            "%{",
            separated_pair(span(ws), tag_or_cut("endif"), span(ws)),
        ),
        |((preamble, trailing), strip)| {
            let mut expr = EndifTemplateExpr::new(strip);
            expr.set_preamble(preamble);
            expr.set_trailing(trailing);
            expr
        },
    );

    map(
        tuple((if_expr, opt(else_expr), endif_expr)),
        |(if_expr, else_expr, endif_expr)| IfDirective::new(if_expr, else_expr, endif_expr),
    )(input)
}

fn for_directive(input: Input) -> IResult<Input, ForDirective> {
    let for_expr = map(
        pair(
            template_tag(
                "%{",
                tuple((
                    pair(terminated(span(ws), tag("for")), decor(ws, cut(ident), ws)),
                    opt(preceded(char(','), decor(ws, cut(ident), ws))),
                    preceded(tag_or_cut("in"), decor(ws, cut(expr), ws)),
                )),
            ),
            spanned(template),
        ),
        |((((preamble, key_var), value_var, collection_expr), strip), template)| {
            let (key_var, value_var) = match value_var {
                Some(value_var) => (Some(key_var), value_var),
                None => (None, key_var),
            };

            let mut expr =
                ForTemplateExpr::new(key_var, value_var, collection_expr, template, strip);
            expr.set_preamble(preamble);
            expr
        },
    );

    let endfor_expr = map(
        template_tag(
            "%{",
            separated_pair(span(ws), tag_or_cut("endfor"), span(ws)),
        ),
        |((preamble, trailing), strip)| {
            let mut expr = EndforTemplateExpr::new(strip);
            expr.set_preamble(preamble);
            expr.set_trailing(trailing);
            expr
        },
    );

    map(pair(for_expr, endfor_expr), |(for_expr, endfor_expr)| {
        ForDirective::new(for_expr, endfor_expr)
    })(input)
}

fn directive(input: Input) -> IResult<Input, Directive> {
    alt((
        map(if_directive, |d| Directive::If(d.into())),
        map(for_directive, |d| Directive::For(d.into())),
    ))(input)
}

fn build_template<'a, F>(literal: F) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Template>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, InternalString>,
{
    map(
        many0(spanned(alt((
            map(literal, |s| Element::Literal(s.into())),
            map(interpolation, |i| Element::Interpolation(i.into())),
            map(directive, Element::Directive),
        )))),
        Template::new,
    )
}

pub fn quoted_string_template(input: Input) -> IResult<Input, Template> {
    delimited(
        char('"'),
        build_template(build_string(string_fragment(string_literal))),
        char('"'),
    )(input)
}

pub fn heredoc_template(input: Input) -> IResult<Input, Template> {
    build_template(map(literal(alt((tag("${"), tag("%{")))), Into::into))(input)
}

pub fn template(input: Input) -> IResult<Input, Template> {
    build_template(build_string(string_fragment(literal(alt((
        tag("\\"),
        tag("${"),
        tag("%{"),
    ))))))(input)
}
