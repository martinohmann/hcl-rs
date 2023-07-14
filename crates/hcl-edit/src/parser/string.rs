use super::{trivia::void, Input};
use crate::{Decorated, Ident, RawString};
use std::borrow::Cow;
use winnow::{
    combinator::{alt, cut_err, delimited, fail, not, opt, preceded, repeat, success},
    dispatch,
    error::{ContextError, StrContext, StrContextValue},
    stream::AsChar,
    token::{any, one_of, take, take_while},
    PResult, Parser,
};

pub(super) fn string(input: &mut Input) -> PResult<String> {
    delimited(b'"', opt(build_string(quoted_string_fragment)), b'"')
        .map(Option::unwrap_or_default)
        .output_into()
        .parse_next(input)
}

pub(super) fn build_string<'a, F>(
    mut fragment_parser: F,
) -> impl Parser<Input<'a>, Cow<'a, str>, ContextError>
where
    F: Parser<Input<'a>, StringFragment<'a>, ContextError>,
{
    move |input: &mut Input<'a>| {
        let mut string = match fragment_parser.parse_next(input) {
            Ok(fragment) => match fragment {
                StringFragment::Literal(s) => Cow::Borrowed(s),
                StringFragment::EscapedChar(c) => Cow::Owned(String::from(c)),
                StringFragment::EscapedMarker(m) => Cow::Borrowed(m.unescape()),
            },
            Err(err) => return Err(err),
        };

        loop {
            match fragment_parser.parse_next(input) {
                Ok(fragment) => {
                    match fragment {
                        StringFragment::Literal(s) => string.to_mut().push_str(s),
                        StringFragment::EscapedChar(c) => string.to_mut().push(c),
                        StringFragment::EscapedMarker(m) => string.to_mut().push_str(m.unescape()),
                    };
                }
                Err(_) => return Ok(string),
            }
        }
    }
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character or an escaped template start marker.
#[derive(Clone)]
pub(super) enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedMarker(EscapedMarker),
}

/// An escaped marker which would start a template interpolation or directive if unescaped.
#[derive(Clone)]
pub(super) enum EscapedMarker {
    Interpolation,
    Directive,
}

impl EscapedMarker {
    // Returns the unescaped form of the escaped marker.
    fn unescape(&self) -> &'static str {
        match self {
            EscapedMarker::Interpolation => "${",
            EscapedMarker::Directive => "%{",
        }
    }
}

pub(super) fn quoted_string_fragment<'a>(input: &mut Input<'a>) -> PResult<StringFragment<'a>> {
    alt((
        escaped_marker.map(StringFragment::EscapedMarker),
        string_literal.map(StringFragment::Literal),
        escaped_char.map(StringFragment::EscapedChar),
    ))
    .parse_next(input)
}

pub(super) fn template_string_fragment<'a, F, T>(
    mut literal_end: F,
) -> impl Parser<Input<'a>, StringFragment<'a>, ContextError>
where
    F: Parser<Input<'a>, T, ContextError>,
{
    move |input: &mut Input<'a>| {
        alt((
            escaped_marker.map(StringFragment::EscapedMarker),
            any_until(literal_end.by_ref()).map(StringFragment::Literal),
        ))
        .parse_next(input)
    }
}

/// Parse a non-empty block of text that doesn't include `"` or non-escaped template
/// interpolation/directive start markers.
fn string_literal<'a>(input: &mut Input<'a>) -> PResult<&'a str> {
    let literal_end = dispatch! {any;
        b'\"' | b'\\' => success(true),
        b'$' | b'%' => b'{'.value(true),
        _ => fail,
    };
    any_until(literal_end).parse_next(input)
}

fn any_until<'a, F, T>(literal_end: F) -> impl Parser<Input<'a>, &'a str, ContextError>
where
    F: Parser<Input<'a>, T, ContextError>,
{
    void(repeat(
        1..,
        preceded(not(alt((escaped_marker.void(), literal_end.void()))), any),
    ))
    .recognize()
    .try_map(std::str::from_utf8)
}

/// Parse an escaped start marker for a template interpolation or directive.
fn escaped_marker(input: &mut Input) -> PResult<EscapedMarker> {
    dispatch! {take::<_, Input, _>(3usize);
        b"$${" => success(EscapedMarker::Interpolation),
        b"%%{" => success(EscapedMarker::Directive),
        _ => fail,
    }
    .parse_next(input)
}

/// Parse an escaped character: `\n`, `\t`, `\r`, `\u00AC`, etc.
fn escaped_char(input: &mut Input) -> PResult<char> {
    b'\\'.parse_next(input)?;

    dispatch! {any;
        b'n' => success('\n'),
        b'r' => success('\r'),
        b't' => success('\t'),
        b'\\' => success('\\'),
        b'"' => success('"'),
        b'/' => success('/'),
        b'b' => success('\u{08}'),
        b'f' => success('\u{0C}'),
        b'u' => cut_err(hexescape::<4>)
            .context(StrContext::Label("unicode 4-digit hex code")),
        b'U' => cut_err(hexescape::<8>)
            .context(StrContext::Label("unicode 8-digit hex code")),
        _ => cut_err(fail)
            .context(StrContext::Label("escape sequence"))
            .context(StrContext::Expected(StrContextValue::CharLiteral('b')))
            .context(StrContext::Expected(StrContextValue::CharLiteral('f')))
            .context(StrContext::Expected(StrContextValue::CharLiteral('n')))
            .context(StrContext::Expected(StrContextValue::CharLiteral('r')))
            .context(StrContext::Expected(StrContextValue::CharLiteral('t')))
            .context(StrContext::Expected(StrContextValue::CharLiteral('u')))
            .context(StrContext::Expected(StrContextValue::CharLiteral('U')))
            .context(StrContext::Expected(StrContextValue::CharLiteral('\\')))
            .context(StrContext::Expected(StrContextValue::CharLiteral('"'))),
    }
    .parse_next(input)
}

fn hexescape<const N: usize>(input: &mut Input) -> PResult<char> {
    let parse_hex =
        take_while(1..=N, |c: u8| c.is_ascii_hexdigit()).verify(|hex: &[u8]| hex.len() == N);

    let parse_u32 = parse_hex.try_map(|hex: &[u8]| {
        u32::from_str_radix(
            unsafe { from_utf8_unchecked(hex, "`is_ascii_hexdigit` filters out non-ascii") },
            16,
        )
    });

    parse_u32.verify_map(std::char::from_u32).parse_next(input)
}

pub(super) fn raw_string<'a, P, O>(inner: P) -> impl Parser<Input<'a>, RawString, ContextError>
where
    P: Parser<Input<'a>, O, ContextError>,
{
    inner.span().map(RawString::from_span)
}

pub(super) fn ident(input: &mut Input) -> PResult<Decorated<Ident>> {
    str_ident
        .map(|ident| Decorated::new(Ident::new_unchecked(ident)))
        .parse_next(input)
}

pub(super) fn str_ident<'a>(input: &mut Input<'a>) -> PResult<&'a str> {
    (one_of(is_id_start), take_while(0.., is_id_continue))
        .recognize()
        .map(|s: &[u8]| unsafe {
            from_utf8_unchecked(s, "`is_id_start` and `is_id_continue` filter out non-utf8")
        })
        .parse_next(input)
}

#[inline]
pub(super) fn is_id_start(b: u8) -> bool {
    hcl_primitives::ident::is_id_start(b.as_char())
}

#[inline]
fn is_id_continue(b: u8) -> bool {
    hcl_primitives::ident::is_id_continue(b.as_char())
}

pub(super) unsafe fn from_utf8_unchecked<'b>(
    bytes: &'b [u8],
    safety_justification: &'static str,
) -> &'b str {
    if cfg!(debug_assertions) {
        std::str::from_utf8(bytes).expect(safety_justification)
    } else {
        std::str::from_utf8_unchecked(bytes)
    }
}
