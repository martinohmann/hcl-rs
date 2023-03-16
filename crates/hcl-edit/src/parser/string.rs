use super::{
    context::{Context, Expected},
    error::ParseError,
    trivia::void,
    IResult, Input,
};
use crate::{repr::Decorated, Ident, InternalString, RawString};
use std::borrow::Cow;
use winnow::{
    branch::alt,
    bytes::{any, one_of, tag, take_while0, take_while_m_n},
    combinator::{cut_err, fail, not, opt, success},
    dispatch,
    multi::many1,
    sequence::{delimited, preceded},
    stream::AsChar,
    Parser,
};

pub(super) fn string(input: Input) -> IResult<Input, InternalString> {
    delimited(b'"', opt(build_string), b'"')
        .map(Option::unwrap_or_default)
        .parse_next(input)
}

pub(super) fn build_string(input: Input) -> IResult<Input, InternalString> {
    let (mut input, mut string) = match string_fragment(input) {
        Ok((input, fragment)) => match fragment {
            StringFragment::Literal(s) => (input, Cow::Borrowed(s)),
            StringFragment::EscapedChar(c) => (input, Cow::Owned(String::from(c))),
        },
        Err(err) => return Err(err),
    };

    loop {
        match string_fragment(input) {
            Ok((rest, fragment)) => {
                match fragment {
                    StringFragment::Literal(s) => string.to_mut().push_str(s),
                    StringFragment::EscapedChar(c) => string.to_mut().push(c),
                };
                input = rest;
            }
            Err(_) => return Ok((input, string.into())),
        }
    }
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters) or a single
/// parsed escaped character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
}

fn string_fragment(input: Input) -> IResult<Input, StringFragment> {
    alt((
        string_literal.map(StringFragment::Literal),
        escaped_char.map(StringFragment::EscapedChar),
    ))(input)
}

/// Parse a non-empty block of text that doesn't include `\`,  `"` or non-escaped template
/// interpolation/directive start markers.
fn string_literal(input: Input) -> IResult<Input, &str> {
    let literal_end = dispatch! {any;
        b'\"' | b'\\' => success(true),
        b'$' | b'%' => one_of(b'{').value(true),
        _ => fail,
    };
    literal_until(literal_end).parse_next(input)
}

pub(super) fn literal_until<'a, F, T>(
    literal_end: F,
) -> impl Parser<Input<'a>, &'a str, ParseError<Input<'a>>>
where
    F: Parser<Input<'a>, T, ParseError<Input<'a>>>,
{
    void(many1(alt((
        tag("$${"),
        tag("%%{"),
        preceded(not(literal_end), any).recognize(),
    ))))
    .recognize()
    .map_res(std::str::from_utf8)
}

/// Parse an escaped character: `\n`, `\t`, `\r`, `\u00AC`, etc.
fn escaped_char(input: Input) -> IResult<Input, char> {
    let (input, _) = b'\\'.parse_next(input)?;

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
            .context(Context::Expression("unicode 4-digit hex code")),
        b'U' => cut_err(hexescape::<8>)
            .context(Context::Expression("unicode 8-digit hex code")),
        _ => cut_err(fail)
            .context(Context::Expression("escape sequence"))
            .context(Context::Expected(Expected::Char('b')))
            .context(Context::Expected(Expected::Char('f')))
            .context(Context::Expected(Expected::Char('n')))
            .context(Context::Expected(Expected::Char('r')))
            .context(Context::Expected(Expected::Char('t')))
            .context(Context::Expected(Expected::Char('u')))
            .context(Context::Expected(Expected::Char('U')))
            .context(Context::Expected(Expected::Char('\\')))
            .context(Context::Expected(Expected::Char('"'))),
    }
    .parse_next(input)
}

fn hexescape<const N: usize>(input: Input) -> IResult<Input, char> {
    let parse_hex =
        take_while_m_n(1, N, |c: u8| c.is_ascii_hexdigit()).verify(|hex: &[u8]| hex.len() == N);

    let parse_u32 = parse_hex.map_res(|hex: &[u8]| {
        u32::from_str_radix(
            unsafe { from_utf8_unchecked(hex, "`is_ascii_hexdigit` filters out non-ascii") },
            16,
        )
    });

    parse_u32.verify_map(std::char::from_u32).parse_next(input)
}

pub(super) fn raw_string<'a, P, O>(
    inner: P,
) -> impl Parser<Input<'a>, RawString, ParseError<Input<'a>>>
where
    P: Parser<Input<'a>, O, ParseError<Input<'a>>>,
{
    inner.span().map(RawString::from_span)
}

pub(super) fn ident(input: Input) -> IResult<Input, Decorated<Ident>> {
    str_ident
        .map(|ident| Decorated::new(Ident::new_unchecked(ident)))
        .parse_next(input)
}

pub(super) fn str_ident(input: Input) -> IResult<Input, &str> {
    (one_of(is_id_start), take_while0(is_id_continue))
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
