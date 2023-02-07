use super::ast::{
    Directive, Element, Expression, ForDirective, IfDirective, Interpolation, Template,
};
use super::repr::{Decorated, Spanned};
use super::{
    char_or_cut, decor, expr::expr, ident, literal, spanned, string_fragment, string_literal,
    tag_or_cut, ws, IResult, Input, StringFragment,
};
use crate::template::StripMode;
use crate::Identifier;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    multi::{fold_many1, many0},
    sequence::{delimited, pair, preceded, terminated, tuple},
};

fn build_literal<'a, F>(literal: F) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, String>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, &'a str>,
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

fn interpolation(input: Input) -> IResult<Input, Interpolation> {
    map(
        template_tag("${", decor(ws, cut(expr), ws)),
        |(expr, strip)| Interpolation { expr, strip },
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
    struct IfExpr {
        cond_expr: Expression,
        template: Spanned<Template>,
        strip: StripMode,
    }

    #[derive(Default)]
    struct ElseExpr {
        template: Option<Spanned<Template>>,
        strip: StripMode,
    }

    let if_expr = map(
        pair(
            template_tag(
                "%{",
                preceded(pair(ws, tag("if")), decor(ws, cut(expr), ws)),
            ),
            spanned(template),
        ),
        |((cond_expr, strip), template)| IfExpr {
            cond_expr,
            template,
            strip,
        },
    );

    let else_expr = map(
        pair(
            template_tag("%{", delimited(ws, tag("else"), ws)),
            spanned(template),
        ),
        |((_, strip), template)| ElseExpr {
            template: Some(template),
            strip,
        },
    );

    let endif_expr = map(
        template_tag("%{", delimited(ws, tag_or_cut("endif"), ws)),
        |(_, strip)| strip,
    );

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

fn for_directive(input: Input) -> IResult<Input, ForDirective> {
    struct ForExpr {
        key_var: Option<Decorated<Identifier>>,
        value_var: Decorated<Identifier>,
        collection_expr: Expression,
        template: Spanned<Template>,
        strip: StripMode,
    }

    let for_expr = map(
        pair(
            template_tag(
                "%{",
                tuple((
                    preceded(pair(ws, tag("for")), decor(ws, cut(ident), ws)),
                    opt(preceded(char(','), decor(ws, cut(ident), ws))),
                    preceded(tag_or_cut("in"), decor(ws, cut(expr), ws)),
                )),
            ),
            spanned(template),
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

    let endfor_expr = map(
        template_tag("%{", delimited(ws, tag_or_cut("endfor"), ws)),
        |(_, strip)| strip,
    );

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

fn directive(input: Input) -> IResult<Input, Directive> {
    alt((
        map(if_directive, |d| Directive::If(d.into())),
        map(for_directive, |d| Directive::For(d.into())),
    ))(input)
}

fn build_template<'a, F>(literal: F) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Template>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, String>,
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
        build_template(build_literal(string_literal)),
        char('"'),
    )(input)
}

pub fn heredoc_template(input: Input) -> IResult<Input, Template> {
    build_template(map(literal(alt((tag("${"), tag("%{")))), |s| s.to_string()))(input)
}

pub fn template(input: Input) -> IResult<Input, Template> {
    build_template(build_literal(literal(alt((
        tag("\\"),
        tag("${"),
        tag("%{"),
    )))))(input)
}
