pub mod ast;
mod encode;
mod error;
mod escape;
mod expr;
mod input;
pub mod repr;
mod structure;
mod template;
#[cfg(test)]
mod tests;

pub use self::ast::*;
use self::error::{Context, Expected, IResult, InternalError};
pub use self::error::{Error, ParseResult};
use self::input::Input;
use self::repr::{Decorate, Decorated, RawString, SetSpan};
use self::structure::body;
use self::template::template;
use crate::{Identifier, InternalString, Number};
use std::borrow::Cow;
use std::str::FromStr;
use winnow::stream::AsChar;
use winnow::{
    branch::alt,
    bytes::{any, one_of, tag, take_until0, take_while_m_n},
    character::{alpha1, alphanumeric1, digit1, multispace0, not_line_ending, space0},
    combinator::{cut_err, fail, not, opt, peek, success},
    dispatch,
    multi::{many0, many1},
    prelude::*,
    sequence::{delimited, preceded, terminated},
    Parser,
};

/// Parse a `hcl::Body` from a `&str`.
///
/// If deserialization into a different type is preferred consider using [`hcl::from_str`][crate::from_str].
///
/// # Example
///
/// ```
/// use hcl::{Attribute, Block, Body};
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let input = r#"
///     some_attr = "foo"
///
///     some_block "some_block_label" {
///       attr = "value"
///     }
/// "#;
///
/// let expected = Body::builder()
///     .add_attribute(("some_attr", "foo"))
///     .add_block(
///         Block::builder("some_block")
///             .add_label("some_block_label")
///             .add_attribute(("attr", "value"))
///             .build()
///     )
///     .build();
///
/// let body = hcl::parse(input)?;
///
/// assert_eq!(body, expected);
/// #   Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This function fails with an error if the `input` cannot be parsed as HCL.
pub fn parse(input: &str) -> ParseResult<crate::structure::Body> {
    parse_raw(input).map(Into::into)
}

#[allow(missing_docs)]
#[allow(clippy::missing_errors_doc)]
pub fn parse_raw(input: &str) -> ParseResult<Body> {
    parse_to_end(input, body)
}

pub(crate) fn parse_template(input: &str) -> ParseResult<crate::template::Template> {
    parse_to_end(input, template).map(Into::into)
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
    cut_err(tag(t)).context(Context::Expected(Expected::Literal(t)))
}

fn hash_line_comment(input: Input) -> IResult<Input, ()> {
    preceded(b'#', not_line_ending).void().parse_next(input)
}

fn double_slash_line_comment(input: Input) -> IResult<Input, ()> {
    preceded(tag("//"), not_line_ending)
        .void()
        .parse_next(input)
}

fn inline_comment(input: Input) -> IResult<Input, ()> {
    delimited(tag("/*"), take_until0("*/"), tag("*/"))
        .void()
        .parse_next(input)
}

fn line_comment(input: Input) -> IResult<Input, ()> {
    let (input, ch) = peek(any)(input)?;

    match ch {
        b'#' => cut_err(hash_line_comment)(input),
        b'/' => cut_err(double_slash_line_comment)(input),
        _ => fail(input),
    }
}

fn comment(input: Input) -> IResult<Input, ()> {
    let (input, ch) = peek(any)(input)?;

    match ch {
        b'#' => cut_err(hash_line_comment)(input),
        b'/' => cut_err(alt((double_slash_line_comment, inline_comment)))(input),
        _ => fail(input),
    }
}

fn sp(input: Input) -> IResult<Input, ()> {
    (space0.void(), void(many0((inline_comment, space0.void()))))
        .void()
        .parse_next(input)
}

fn spc(input: Input) -> IResult<Input, ()> {
    (
        space0.void(),
        void(many0((inline_comment, space0.void()))),
        opt(line_comment),
    )
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

fn spanned<'a, F, O>(inner: F) -> impl Parser<Input<'a>, O, InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, O, InternalError<Input<'a>>>,
    O: SetSpan,
{
    inner.with_span().map(|(mut value, span)| {
        value.set_span(span);
        value
    })
}

fn prefix_decor<'a, F, G, O1, O2>(
    prefix: F,
    inner: G,
) -> impl Parser<Input<'a>, O2, InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, O1, InternalError<Input<'a>>>,
    G: Parser<Input<'a>, O2, InternalError<Input<'a>>>,
    O2: Decorate + SetSpan,
{
    (raw(prefix), inner.with_span()).map(|(prefix, (mut value, span))| {
        value.decor_mut().set_prefix(prefix);
        value.set_span(span);
        value
    })
}

fn suffix_decor<'a, F, G, O1, O2>(
    inner: F,
    suffix: G,
) -> impl Parser<Input<'a>, O1, InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, O1, InternalError<Input<'a>>>,
    G: Parser<Input<'a>, O2, InternalError<Input<'a>>>,
    O1: Decorate + SetSpan,
{
    (inner.with_span(), raw(suffix)).map(|((mut value, span), suffix)| {
        value.decor_mut().set_suffix(suffix);
        value.set_span(span);
        value
    })
}

fn decor<'a, F, G, H, O1, O2, O3>(
    prefix: F,
    inner: G,
    suffix: H,
) -> impl Parser<Input<'a>, O2, InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, O1, InternalError<Input<'a>>>,
    G: Parser<Input<'a>, O2, InternalError<Input<'a>>>,
    H: Parser<Input<'a>, O3, InternalError<Input<'a>>>,
    O2: Decorate + SetSpan,
{
    (raw(prefix), inner.with_span(), raw(suffix)).map(|(prefix, (mut value, span), suffix)| {
        let decor = value.decor_mut();
        decor.set_prefix(prefix);
        decor.set_suffix(suffix);
        value.set_span(span);
        value
    })
}

fn raw<'a, P, O>(inner: P) -> impl Parser<Input<'a>, RawString, InternalError<Input<'a>>>
where
    P: Parser<Input<'a>, O, InternalError<Input<'a>>>,
{
    inner.span().map(RawString::from_span)
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
        b'u' => cut_err(hexescape::<4>).context(Context::Expression("unicode 4-digit hex code")),
        b'U' => cut_err(hexescape::<8>).context(Context::Expression("unicode 8-digit hex code")),
        _ => {
            cut_err(fail)
            .context(Context::Expression("escape sequence"))
            .context(Context::Expected(Expected::Char('b')))
            .context(Context::Expected(Expected::Char('f')))
            .context(Context::Expected(Expected::Char('n')))
            .context(Context::Expected(Expected::Char('r')))
            .context(Context::Expected(Expected::Char('t')))
            .context(Context::Expected(Expected::Char('u')))
            .context(Context::Expected(Expected::Char('U')))
            .context(Context::Expected(Expected::Char('\\')))
            .context(Context::Expected(Expected::Char('"')))
        }
    }
    .parse_next(input)
}

/// Parse a non-empty block of text that doesn't include `\`,  `"` or non-escaped template
/// interpolation/directive start markers.
fn string_literal(input: Input) -> IResult<Input, &str> {
    literal(alt((one_of("\"\\").recognize(), tag("${"), tag("%{")))).parse_next(input)
}

fn literal<'a, F, T>(literal_end: F) -> impl Parser<Input<'a>, &'a str, InternalError<Input<'a>>>
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

fn str_ident(input: Input) -> IResult<Input, &str> {
    (
        alt((alpha1, tag("_"))),
        void(many0(alt((alphanumeric1, tag("_"), tag("-"))))),
    )
        .recognize()
        .map(|s: &[u8]| unsafe {
            from_utf8_unchecked(s, "`alpha1` and `alphanumeric1` filter out non-ascii")
        })
        .parse_next(input)
}

fn cut_ident(input: Input) -> IResult<Input, Decorated<Identifier>> {
    cut_err(ident)
        .context(Context::Expected(Expected::Description("identifier")))
        .parse_next(input)
}

fn ident(input: Input) -> IResult<Input, Decorated<Identifier>> {
    str_ident
        .map(|ident| Decorated::new(Identifier::unchecked(ident)))
        .parse_next(input)
}

fn exponent(input: Input) -> IResult<Input, &[u8]> {
    (one_of("eE"), opt(one_of("+-")), cut_err(digit1))
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

#[inline(always)]
fn void<'a, P>(inner: P) -> impl Parser<Input<'a>, (), InternalError<Input<'a>>>
where
    P: Parser<Input<'a>, (), InternalError<Input<'a>>>,
{
    inner
}
