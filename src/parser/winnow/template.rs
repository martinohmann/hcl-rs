use super::ast::{
    Directive, Element, ElseTemplateExpr, EndforTemplateExpr, EndifTemplateExpr, ForDirective,
    ForTemplateExpr, IfDirective, IfTemplateExpr, Interpolation, Template,
};
use super::error::InternalError;
use super::StringTemplate;
use super::{
    build_string, cut_char, cut_ident, cut_tag, decor, expr::expr, literal, repr::Span, spanned,
    string_fragment, string_literal, ws, IResult, Input,
};
use crate::template::StripMode;
use crate::InternalString;
use winnow::sequence::separated_pair;
use winnow::Parser;
use winnow::{
    branch::alt,
    bytes::tag,
    character::{line_ending, space0},
    combinator::opt,
    multi::many0,
    sequence::{delimited, preceded, terminated},
};

fn interpolation(input: Input) -> IResult<Input, Interpolation> {
    template_tag("${", decor(ws, expr, ws))
        .map(|(expr, strip)| Interpolation::new(expr, strip))
        .parse_next(input)
}

fn template_tag<'a, P, O>(
    start_tag: &'a str,
    inner: P,
) -> impl Parser<Input<'a>, (O, StripMode), InternalError<Input<'a>>>
where
    P: Parser<Input<'a>, O, InternalError<Input<'a>>>,
{
    (
        preceded(tag(start_tag), opt(b'~')),
        inner,
        terminated(opt(b'~'), cut_char('}')),
    )
        .map(|(strip_start, output, strip_end)| {
            (output, (strip_start.is_some(), strip_end.is_some()).into())
        })
}

fn if_directive(input: Input) -> IResult<Input, IfDirective> {
    let if_expr = (
        template_tag(
            "%{",
            (terminated(ws.span(), tag("if")), decor(ws, expr, ws)),
        ),
        spanned(template),
    )
        .map(|(((preamble, cond_expr), strip), template)| {
            let mut expr = IfTemplateExpr::new(cond_expr, template, strip);
            expr.set_preamble(preamble);
            expr
        });

    let else_expr = (
        template_tag("%{", separated_pair(ws.span(), tag("else"), ws.span())),
        spanned(template),
    )
        .map(|(((preamble, trailing), strip), template)| {
            let mut expr = ElseTemplateExpr::new(template, strip);
            expr.set_preamble(preamble);
            expr.set_trailing(trailing);
            expr
        });

    let endif_expr = template_tag("%{", separated_pair(ws.span(), cut_tag("endif"), ws.span()))
        .map(|((preamble, trailing), strip)| {
            let mut expr = EndifTemplateExpr::new(strip);
            expr.set_preamble(preamble);
            expr.set_trailing(trailing);
            expr
        });

    (if_expr, opt(else_expr), endif_expr)
        .map(|(if_expr, else_expr, endif_expr)| IfDirective::new(if_expr, else_expr, endif_expr))
        .parse_next(input)
}

fn for_directive(input: Input) -> IResult<Input, ForDirective> {
    let for_expr = (
        template_tag(
            "%{",
            (
                (terminated(ws.span(), tag("for")), decor(ws, cut_ident, ws)),
                opt(preceded(b',', decor(ws, cut_ident, ws))),
                preceded(cut_tag("in"), decor(ws, expr, ws)),
            ),
        ),
        spanned(template),
    )
        .map(
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

    let endfor_expr = template_tag(
        "%{",
        separated_pair(ws.span(), cut_tag("endfor"), ws.span()),
    )
    .map(|((preamble, trailing), strip)| {
        let mut expr = EndforTemplateExpr::new(strip);
        expr.set_preamble(preamble);
        expr.set_trailing(trailing);
        expr
    });

    (for_expr, endfor_expr)
        .map(|(for_expr, endfor_expr)| ForDirective::new(for_expr, endfor_expr))
        .parse_next(input)
}

fn directive(input: Input) -> IResult<Input, Directive> {
    alt((
        if_directive.map(Directive::If),
        for_directive.map(Directive::For),
    ))(input)
}

fn elements<'a, P>(literal: P) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Vec<Element>>
where
    P: Parser<Input<'a>, InternalString, InternalError<Input<'a>>>,
{
    many0(spanned(alt((
        literal.map(|s| Element::Literal(s.into())),
        interpolation.map(Element::Interpolation),
        directive.map(Element::Directive),
    ))))
}

fn build_template<'a, P>(literal: P) -> impl Parser<Input<'a>, Template, InternalError<Input<'a>>>
where
    P: Parser<Input<'a>, InternalString, InternalError<Input<'a>>>,
{
    elements(literal).map(Template::new)
}

fn build_string_template<'a, F>(
    literal: F,
) -> impl Parser<Input<'a>, StringTemplate, InternalError<Input<'a>>>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, InternalString>,
{
    elements(literal).map(StringTemplate::new)
}

pub fn string_template(input: Input) -> IResult<Input, StringTemplate> {
    delimited(
        b'"',
        build_string_template(build_string(string_fragment(string_literal))),
        b'"',
    )(input)
}

pub fn template(input: Input) -> IResult<Input, Template> {
    build_template(literal(alt((tag("${"), tag("%{")))).output_into()).parse_next(input)
}

pub fn heredoc_template<'a>(
    delim: &'a str,
) -> impl Parser<Input<'a>, Template, InternalError<Input<'a>>> {
    move |input: Input<'a>| {
        // We'll need to look for a newline character followed by optional space and the heredoc
        // delimiter.
        //
        // Here's the catch though: the newline character has to be treated as part of
        // the last template literal, but is still required to be matched when detecting the
        // heredoc end. Matching only `(space0, tag(delim))` is not sufficient, because heredocs
        // can contain their delimiter as long as it's not preceeded by an newline character.
        //
        // Handling this case via parser combinators is quite tricky and thus we'll manually add
        // the newline character to the last template element below.
        let heredoc_end = (line_ending, space0, tag(delim)).recognize();
        let literal_end = alt((tag("${"), tag("%{"), heredoc_end));
        let elements = elements(literal(literal_end).output_into());

        // Use `opt` to handle an empty template.
        opt(
            (elements, line_ending.span()).map(|(mut elements, newline_span)| {
                // If there is a trailing literal, update its span and append the newline character to it.
                // Otherwise just add a new literal containing only the newline character.
                if let Some(Element::Literal(lit)) = elements.last_mut() {
                    let span = lit.span().unwrap();
                    lit.set_span(span.start..newline_span.end);
                    let mut existing = String::from(lit.as_str());
                    existing.push('\n');
                    *lit.as_mut() = existing.into();
                } else {
                    let newline =
                        Element::Literal(InternalString::from("\n").into()).spanned(newline_span);
                    elements.push(newline);
                }

                Template::new(elements)
            }),
        )
        .map(Option::unwrap_or_default)
        .parse_next(input)
    }
}
