#![allow(missing_docs)]

mod error;
mod expr;
mod repr;
mod structure;
mod template;
#[cfg(test)]
mod tests;

use self::error::{Context, Expected, IResult, InternalError};
pub use self::error::{Error, ParseResult};
use self::expr::expr;
use self::repr::{decor, prefix_decor, raw, spanned, suffix_decor};
use self::structure::body;
use self::template::template;
use crate::expr::Expression;
use crate::repr::{Decorated, Despan};
use crate::structure::Body;
use crate::template::Template;
use crate::{Ident, InternalString, Number};
use hcl_primitives::ident;
use std::borrow::Cow;
use std::str::FromStr;
use winnow::{
    branch::alt,
    bytes::{any, one_of, tag, take_until0, take_while0, take_while_m_n},
    character::{digit1, multispace0, not_line_ending, space0},
    combinator::{cut_err, fail, not, opt, peek, success},
    dispatch,
    multi::{many0, many1},
    prelude::*,
    sequence::{delimited, preceded, terminated},
    stream::{AsChar, Located},
    Parser,
};

pub(crate) type Input<'a> = Located<&'a [u8]>;

pub fn parse_body(input: &str) -> ParseResult<Body> {
    let mut body = parse_to_end(input, body)?;
    body.despan(input);
    Ok(body)
}

pub fn parse_expr(input: &str) -> ParseResult<Expression> {
    let mut expr = parse_to_end(input, expr)?;
    expr.despan(input);
    Ok(expr)
}

pub fn parse_template(input: &str) -> ParseResult<Template> {
    let mut template = parse_to_end(input, template)?;
    template.despan(input);
    Ok(template)
}

fn parse_to_end<'a, F, O>(input: &'a str, mut parser: F) -> ParseResult<O>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
{
    let input = Input::new(input.as_bytes());
    parser
        .parse_next(input)
        .finish()
        .map_err(|err| Error::from_internal_error(input, err))
}

pub(crate) unsafe fn from_utf8_unchecked<'b>(
    bytes: &'b [u8],
    safety_justification: &'static str,
) -> &'b str {
    if cfg!(debug_assertions) {
        std::str::from_utf8(bytes).expect(safety_justification)
    } else {
        std::str::from_utf8_unchecked(bytes)
    }
}

fn cut_char<'a>(c: char) -> impl Parser<Input<'a>, char, InternalError<Input<'a>>> {
    cut_err(one_of(c))
        .map(AsChar::as_char)
        .context(Context::Expected(Expected::Char(c)))
}

fn cut_tag<'a>(t: &'static str) -> impl Parser<Input<'a>, &'a [u8], InternalError<Input<'a>>> {
    cut_err(t).context(Context::Expected(Expected::Literal(t)))
}

fn hash_line_comment(input: Input) -> IResult<Input, ()> {
    preceded(b'#', not_line_ending).void().parse_next(input)
}

fn double_slash_line_comment(input: Input) -> IResult<Input, ()> {
    preceded(b"//", not_line_ending).void().parse_next(input)
}

fn inline_comment(input: Input) -> IResult<Input, ()> {
    delimited(b"/*", take_until0("*/"), b"*/")
        .void()
        .parse_next(input)
}

fn line_comment(input: Input) -> IResult<Input, ()> {
    dispatch! {peek(any);
        b'#' => hash_line_comment,
        b'/' => double_slash_line_comment,
        _ => fail,
    }
    .parse_next(input)
}

fn comment(input: Input) -> IResult<Input, ()> {
    dispatch! {peek(any);
        b'#' => hash_line_comment,
        b'/' => alt((double_slash_line_comment, inline_comment)),
        _ => fail,
    }
    .parse_next(input)
}

fn sp(input: Input) -> IResult<Input, ()> {
    (space0.void(), void(many0((inline_comment, space0.void()))))
        .void()
        .parse_next(input)
}

fn ws(input: Input) -> IResult<Input, ()> {
    (
        multispace0.void(),
        void(many0((comment, multispace0.void()))),
    )
        .void()
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

/// Parse a non-empty block of text that doesn't include `\`,  `"` or non-escaped template
/// interpolation/directive start markers.
fn string_literal(input: Input) -> IResult<Input, &str> {
    let literal_end = alt((b"\"", b"\\", b"${", b"%{"));
    literal_until(literal_end).parse_next(input)
}

fn literal_until<'a, F, T>(
    literal_end: F,
) -> impl Parser<Input<'a>, &'a str, InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, T, InternalError<Input<'a>>>,
{
    void(many1(alt((
        tag("$${"),
        tag("%%{"),
        any_except(literal_end),
    ))))
    .recognize()
    .map_res(std::str::from_utf8)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters) or a single
/// parsed escaped character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
}

fn string_fragment<'a, F>(
    literal: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, StringFragment<'a>>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, &'a str>,
{
    alt((
        literal.map(StringFragment::Literal),
        escaped_char.map(StringFragment::EscapedChar),
    ))
}

fn build_string<'a, F>(
    mut string_fragment: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, InternalString>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, StringFragment<'a>>,
{
    move |input: Input<'a>| {
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
}

fn string(input: Input) -> IResult<Input, InternalString> {
    preceded(
        b'"',
        alt((
            one_of('"').map(|_| InternalString::new()),
            terminated(build_string(string_fragment(string_literal)), b'"'),
        )),
    )(input)
}

#[inline]
fn is_id_start(b: u8) -> bool {
    ident::is_id_start(b.as_char())
}

#[inline]
fn is_id_continue(b: u8) -> bool {
    ident::is_id_continue(b.as_char())
}

fn str_ident(input: Input) -> IResult<Input, &str> {
    (one_of(is_id_start), take_while0(is_id_continue))
        .recognize()
        .map(|s: &[u8]| unsafe {
            from_utf8_unchecked(s, "`alpha1` and `alphanumeric1` filter out non-ascii")
        })
        .parse_next(input)
}

fn cut_ident(input: Input) -> IResult<Input, Decorated<Ident>> {
    cut_err(ident)
        .context(Context::Expected(Expected::Description("identifier")))
        .parse_next(input)
}

fn ident(input: Input) -> IResult<Input, Decorated<Ident>> {
    str_ident
        .map(|ident| Decorated::new(Ident::new_unchecked(ident)))
        .parse_next(input)
}

fn exponent(input: Input) -> IResult<Input, &[u8]> {
    (
        one_of("eE"),
        opt(one_of("+-")),
        cut_err(digit1).context(Context::Expected(Expected::Description("digit"))),
    )
        .recognize()
        .parse_next(input)
}

fn float(input: Input) -> IResult<Input, f64> {
    let fraction = preceded(b'.', digit1);

    terminated(digit1, alt((terminated(fraction, opt(exponent)), exponent)))
        .recognize()
        .map_res(|s: &[u8]| {
            f64::from_str(unsafe {
                from_utf8_unchecked(s, "`digit1` and `exponent` filter out non-ascii")
            })
        })
        .parse_next(input)
}

fn integer(input: Input) -> IResult<Input, u64> {
    digit1
        .map_res(|s: &[u8]| {
            u64::from_str(unsafe { from_utf8_unchecked(s, "`digit1` filters out non-ascii") })
        })
        .parse_next(input)
}

fn number(input: Input) -> IResult<Input, Number> {
    alt((
        float.verify_map(Number::from_f64),
        integer.map(Number::from),
    ))(input)
}

fn any_except<'a, F, T>(inner: F) -> impl Parser<Input<'a>, &'a [u8], InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, T, InternalError<Input<'a>>>,
{
    preceded(not(inner), any).recognize()
}

#[inline]
fn void<'a, P>(inner: P) -> impl Parser<Input<'a>, (), InternalError<Input<'a>>>
where
    P: Parser<Input<'a>, (), InternalError<Input<'a>>>,
{
    inner
}
