use super::{
    context::{cut_char, cut_ident, cut_tag},
    error::ParseError,
    expr::expr,
    repr::{decorated, spanned},
    string::{build_string, from_utf8_unchecked, literal_until, raw_string},
    trivia::ws,
    IResult, Input,
};
use crate::{
    repr::{SetSpan, Span, Spanned},
    template::*,
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
    delimited(b'"', elements(build_string), b'"')
        .output_into()
        .parse_next(input)
}

pub(super) fn template(input: Input) -> IResult<Input, Template> {
    let literal_end = alt((b"${", b"%{"));
    let literal = literal_until(literal_end).output_into();
    elements(literal).output_into().parse_next(input)
}

pub(super) fn heredoc_template<'a>(
    delim: &'a str,
) -> impl Parser<Input<'a>, Template, ParseError<Input<'a>>> {
    move |input: Input<'a>| {
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
        let literal = literal_until(literal_end).output_into();

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

fn elements<'a, P>(literal: P) -> impl Parser<Input<'a>, Vec<Element>, ParseError<Input<'a>>>
where
    P: Parser<Input<'a>, String, ParseError<Input<'a>>>,
{
    many0(spanned(alt((
        literal.map(|s| Element::Literal(Spanned::new(s))),
        interpolation.map(Element::Interpolation),
        directive.map(Element::Directive),
    ))))
}

fn interpolation(input: Input) -> IResult<Input, Interpolation> {
    control(b"${", decorated(ws, expr, ws))
        .map(|(expr, strip)| {
            let mut interp = Interpolation::new(expr);
            interp.set_strip(strip);
            interp
        })
        .parse_next(input)
}

fn directive(input: Input) -> IResult<Input, Directive> {
    alt((
        if_directive.map(Directive::If),
        for_directive.map(Directive::For),
    ))
    .parse_next(input)
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
            let mut expr = IfTemplateExpr::new(cond_expr, template);
            expr.set_strip(strip);
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
            expr.set_strip(strip);
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
        expr.set_strip(strip);
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

                let mut expr = ForTemplateExpr::new(key_var, value_var, collection_expr, template);
                expr.set_strip(strip);
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
        expr.set_strip(strip);
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
            (
                output,
                Strip::from((strip_start.is_some(), strip_end.is_some())),
            )
        })
}
