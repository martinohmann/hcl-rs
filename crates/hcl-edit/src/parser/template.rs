use super::prelude::*;

use super::expr::expr;
use super::repr::{decorated, spanned};
use super::string::{
    build_string, cut_char, cut_ident, cut_tag, from_utf8_unchecked, quoted_string_fragment,
    raw_string, template_string_fragment,
};
use super::trivia::ws;

use crate::template::{
    Directive, Element, ElseTemplateExpr, EndforTemplateExpr, EndifTemplateExpr, ForDirective,
    ForTemplateExpr, IfDirective, IfTemplateExpr, Interpolation, StringTemplate, Strip, Template,
};
use crate::{SetSpan, Span, Spanned};

use std::borrow::Cow;
use winnow::ascii::{line_ending, space0};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated_pair, terminated};

pub(super) fn string_template(input: &mut Input) -> PResult<StringTemplate> {
    delimited(b'"', elements(build_string(quoted_string_fragment)), b'"')
        .output_into()
        .parse_next(input)
}

pub(super) fn template(input: &mut Input) -> PResult<Template> {
    let literal_end = alt((b"${", b"%{"));
    let literal = template_literal(literal_end);
    elements(literal).output_into().parse_next(input)
}

pub(super) fn heredoc_template<'a>(
    delim: &'a str,
) -> impl Parser<Input<'a>, Template, ContextError> {
    move |input: &mut Input<'a>| {
        // We'll need to look for a line ending followed by optional space and the heredoc
        // delimiter.
        //
        // Here's the catch though: the line ending has to be treated as part of the last template
        // literal, but is still required to be matched when detecting the heredoc end. Matching
        // only `(space0, delim)` is not sufficient, because heredocs can contain their delimiter
        // as long as it's not preceeded by a line ending.
        //
        // Handling this case via parser combinators is quite tricky and thus we'll manually add
        // the line ending to the last template element below.
        let heredoc_end = (line_ending, space0, delim).recognize();
        let literal_end = alt((b"${", b"%{", heredoc_end));
        let literal = template_literal(literal_end);

        // Use `opt` to handle an empty template.
        opt((elements(literal), line_ending.with_span()).map(
            |(mut elements, (line_ending, line_ending_span))| {
                let line_ending = unsafe {
                    from_utf8_unchecked(line_ending, "`line_ending` filters out non-ascii")
                };
                // If there is a trailing literal, update its span and append the line ending to
                // it. Otherwise just add a new literal containing only the line ending.
                if let Some(Element::Literal(lit)) = elements.last_mut() {
                    let existing_span = lit.span().unwrap();
                    lit.push_str(line_ending);
                    lit.set_span(existing_span.start..line_ending_span.end);
                } else {
                    let mut lit = Spanned::new(String::from(line_ending));
                    lit.set_span(line_ending_span);
                    elements.push(Element::Literal(lit));
                }

                Template::from(elements)
            },
        ))
        .map(Option::unwrap_or_default)
        .parse_next(input)
    }
}

#[inline]
fn template_literal<'a, F, T>(literal_end: F) -> impl Parser<Input<'a>, Cow<'a, str>, ContextError>
where
    F: Parser<Input<'a>, T, ContextError>,
{
    build_string(template_string_fragment(literal_end))
}

fn elements<'a, P>(literal: P) -> impl Parser<Input<'a>, Vec<Element>, ContextError>
where
    P: Parser<Input<'a>, Cow<'a, str>, ContextError>,
{
    repeat(
        0..,
        spanned(alt((
            literal.map(|s| Element::Literal(Spanned::new(s.into()))),
            interpolation.map(Element::Interpolation),
            directive.map(Element::Directive),
        ))),
    )
}

fn interpolation(input: &mut Input) -> PResult<Interpolation> {
    control(b"${", decorated(ws, expr, ws))
        .map(|(expr, strip)| {
            let mut interp = Interpolation::new(expr);
            interp.strip = strip;
            interp
        })
        .parse_next(input)
}

fn directive(input: &mut Input) -> PResult<Directive> {
    alt((
        if_directive.map(Directive::If),
        for_directive.map(Directive::For),
    ))
    .parse_next(input)
}

fn if_directive(input: &mut Input) -> PResult<IfDirective> {
    let if_expr = (
        control(
            b"%{",
            (terminated(raw_string(ws), b"if"), decorated(ws, expr, ws)),
        ),
        spanned(template),
    )
        .map(|(((preamble, cond_expr), strip), template)| {
            let mut expr = IfTemplateExpr::new(cond_expr, template);
            expr.strip = strip;
            expr.set_preamble(preamble);
            expr
        });

    let else_expr = (
        control(
            b"%{",
            separated_pair(raw_string(ws), b"else", raw_string(ws)),
        ),
        spanned(template),
    )
        .map(|(((preamble, trailing), strip), template)| {
            let mut expr = ElseTemplateExpr::new(template);
            expr.strip = strip;
            expr.set_preamble(preamble);
            expr.set_trailing(trailing);
            expr
        });

    let endif_expr = control(
        b"%{",
        separated_pair(raw_string(ws), cut_tag("endif"), raw_string(ws)),
    )
    .map(|((preamble, trailing), strip)| {
        let mut expr = EndifTemplateExpr::new();
        expr.strip = strip;
        expr.set_preamble(preamble);
        expr.set_trailing(trailing);
        expr
    });

    (if_expr, opt(else_expr), endif_expr)
        .map(|(if_expr, else_expr, endif_expr)| IfDirective::new(if_expr, else_expr, endif_expr))
        .parse_next(input)
}

fn for_directive(input: &mut Input) -> PResult<ForDirective> {
    let for_expr = (
        control(
            b"%{",
            (
                terminated(raw_string(ws), b"for"),
                decorated(ws, cut_ident, ws),
                opt(preceded(b',', decorated(ws, cut_ident, ws))),
                preceded(cut_tag("in"), decorated(ws, expr, ws)),
            ),
        ),
        spanned(template),
    )
        .map(
            |(((preamble, key_var, value_var, collection_expr), strip), template)| {
                let (key_var, value_var) = match value_var {
                    Some(value_var) => (Some(key_var), value_var),
                    None => (None, key_var),
                };

                let mut expr = ForTemplateExpr::new(key_var, value_var, collection_expr, template);
                expr.strip = strip;
                expr.set_preamble(preamble);
                expr
            },
        );

    let endfor_expr = control(
        b"%{",
        separated_pair(raw_string(ws), cut_tag("endfor"), raw_string(ws)),
    )
    .map(|((preamble, trailing), strip)| {
        let mut expr = EndforTemplateExpr::new();
        expr.strip = strip;
        expr.set_preamble(preamble);
        expr.set_trailing(trailing);
        expr
    });

    (for_expr, endfor_expr)
        .map(|(for_expr, endfor_expr)| ForDirective::new(for_expr, endfor_expr))
        .parse_next(input)
}

fn control<'a, S, P, O1, O2>(
    intro: S,
    inner: P,
) -> impl Parser<Input<'a>, (O2, Strip), ContextError>
where
    S: Parser<Input<'a>, O1, ContextError>,
    P: Parser<Input<'a>, O2, ContextError>,
{
    (
        preceded(intro, opt(b'~')),
        inner,
        terminated(opt(b'~'), cut_char('}')),
    )
        .map(|(strip_start, output, strip_end)| {
            (
                output,
                Strip::from((strip_start.is_some(), strip_end.is_some())),
            )
        })
}
