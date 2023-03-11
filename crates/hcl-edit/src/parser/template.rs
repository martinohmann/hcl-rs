use super::{
    context::{cut_char, cut_ident, cut_tag},
    error::ParseError,
    expr::expr,
    repr::{decorated, spanned},
    string::{build_string, literal_until, raw_string, string_fragment, string_literal},
    trivia::ws,
    IResult, Input,
};
use crate::{
    repr::{SetSpan, Span, Spanned},
    template::*,
    InternalString,
};
use winnow::{
    branch::alt,
    character::{line_ending, space0},
    combinator::opt,
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated},
    Parser,
};

pub(super) fn string_template(input: Input) -> IResult<Input, StringTemplate> {
    let literal = build_string(string_fragment(string_literal));
    delimited(b'"', elements(literal).map(StringTemplate::new), b'"')(input)
}

pub(super) fn template(input: Input) -> IResult<Input, Template> {
    let literal_end = alt((b"${", b"%{"));
    let literal = literal_until(literal_end).output_into();
    elements(literal).map(Template::new).parse_next(input)
}

pub(super) fn heredoc_template<'a>(
    delim: &'a str,
) -> impl Parser<Input<'a>, Template, ParseError<Input<'a>>> {
    move |input: Input<'a>| {
        // We'll need to look for a newline character followed by optional space and the heredoc
        // delimiter.
        //
        // Here's the catch though: the newline character has to be treated as part of
        // the last template literal, but is still required to be matched when detecting the
        // heredoc end. Matching only `(space0, delim)` is not sufficient, because heredocs can
        // contain their delimiter as long as it's not preceeded by a newline character.
        //
        // Handling this case via parser combinators is quite tricky and thus we'll manually add
        // the newline character to the last template element below.
        let heredoc_end = (line_ending, space0, delim).recognize();
        let literal_end = alt((b"${", b"%{", heredoc_end));
        let literal = literal_until(literal_end).output_into();

        // Use `opt` to handle an empty template.
        opt(
            (elements(literal), line_ending.span()).map(|(mut elements, newline_span)| {
                // If there is a trailing literal, update its span and append the newline character to it.
                // Otherwise just add a new literal containing only the newline character.
                if let Some(Element::Literal(lit)) = elements.last_mut() {
                    let existing_span = lit.span().unwrap();
                    let mut existing = lit.to_string();
                    existing.push('\n');
                    *lit = Spanned::new(InternalString::from(existing))
                        .spanned(existing_span.start..newline_span.end);
                } else {
                    let lit = Spanned::new(InternalString::from("\n")).spanned(newline_span);
                    elements.push(Element::Literal(lit));
                }

                Template::new(elements)
            }),
        )
        .map(Option::unwrap_or_default)
        .parse_next(input)
    }
}

fn elements<'a, P>(literal: P) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Vec<Element>>
where
    P: Parser<Input<'a>, InternalString, ParseError<Input<'a>>>,
{
    many0(spanned(alt((
        literal.map(|s| Element::Literal(s.into())),
        interpolation.map(Element::Interpolation),
        directive.map(Element::Directive),
    ))))
}

fn interpolation(input: Input) -> IResult<Input, Interpolation> {
    control(b"${", decorated(ws, expr, ws))
        .map(|(expr, strip)| Interpolation::new(expr, strip))
        .parse_next(input)
}

fn directive(input: Input) -> IResult<Input, Directive> {
    alt((
        if_directive.map(Directive::If),
        for_directive.map(Directive::For),
    ))(input)
}

fn if_directive(input: Input) -> IResult<Input, IfDirective> {
    let if_expr = (
        control(
            b"%{",
            (terminated(raw_string(ws), b"if"), decorated(ws, expr, ws)),
        ),
        spanned(template),
    )
        .map(|(((preamble, cond_expr), strip), template)| {
            let mut expr = IfTemplateExpr::new(cond_expr, template, strip);
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
            let mut expr = ElseTemplateExpr::new(template, strip);
            expr.set_preamble(preamble);
            expr.set_trailing(trailing);
            expr
        });

    let endif_expr = control(
        b"%{",
        separated_pair(raw_string(ws), cut_tag("endif"), raw_string(ws)),
    )
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

                let mut expr =
                    ForTemplateExpr::new(key_var, value_var, collection_expr, template, strip);
                expr.set_preamble(preamble);
                expr
            },
        );

    let endfor_expr = control(
        b"%{",
        separated_pair(raw_string(ws), cut_tag("endfor"), raw_string(ws)),
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

fn control<'a, S, P, O1, O2>(
    intro: S,
    inner: P,
) -> impl Parser<Input<'a>, (O2, Strip), ParseError<Input<'a>>>
where
    S: Parser<Input<'a>, O1, ParseError<Input<'a>>>,
    P: Parser<Input<'a>, O2, ParseError<Input<'a>>>,
{
    (
        preceded(intro, opt(b'~')),
        inner,
        terminated(opt(b'~'), cut_char('}')),
    )
        .map(|(strip_start, output, strip_end)| {
            (output, (strip_start.is_some(), strip_end.is_some()).into())
        })
}
