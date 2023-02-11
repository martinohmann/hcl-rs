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
pub use self::error::{Error, ErrorKind, ParseResult};
use self::error::{IResult, InternalError};
use self::input::Input;
use self::repr::{Decorate, Decorated, Span};
use self::structure::body;
use self::template::template;
use crate::{Identifier, Number};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while_m_n},
    character::complete::{
        alpha1, alphanumeric1, anychar, char, digit1, multispace0, not_line_ending, one_of, space0,
    },
    combinator::{all_consuming, cut, map, map_opt, map_res, not, opt, recognize, value, verify},
    error::context,
    multi::{fold_many0, many0_count, many1_count},
    sequence::{delimited, pair, preceded, terminated, tuple},
    AsChar, Compare, CompareResult, Finish, InputIter, InputLength, InputTake, Parser, Slice,
};
use std::ops::Range;
use std::str::FromStr;

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
    parse_raw(input).map(|node| node.into_inner().into())
}

#[allow(missing_docs)]
pub fn parse_raw(input: &str) -> ParseResult<Decorated<Body>> {
    parse_to_end(input, body)
}

pub(crate) fn parse_template(input: &str) -> ParseResult<crate::template::Template> {
    parse_to_end(input, template).map(Into::into)
}

fn parse_to_end<'a, F, O>(input: &'a str, parser: F) -> ParseResult<O>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
{
    let input = Input::new(input.as_bytes());
    all_consuming(parser)
        .parse(input)
        .finish()
        .map(|(_, output)| output)
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

fn char_or_cut(c: char) -> impl Fn(Input) -> IResult<Input, char> {
    move |input: Input| match input.iter_elements().next().map(|t| {
        let b = t.as_char() == c;
        (&c, b)
    }) {
        Some((c, true)) => Ok((input.slice(c.len()..), c.as_char())),
        _ => Err(nom::Err::Failure(InternalError::new(
            input,
            ErrorKind::Char(c),
        ))),
    }
}

fn tag_or_cut<'a>(tag: &'a str) -> impl Fn(Input<'a>) -> IResult<Input<'a>, Input<'a>> {
    move |input: Input<'a>| {
        let tag_len = tag.input_len();
        match input.compare(tag) {
            CompareResult::Ok => {
                let (input, tag) = input.take_split(tag_len);
                Ok((input, tag))
            }
            _ => Err(nom::Err::Failure(InternalError::new(
                input,
                ErrorKind::Tag(Input::new(tag.as_bytes())),
            ))),
        }
    }
}

fn line_comment(input: Input) -> IResult<Input, ()> {
    value((), pair(alt((tag("#"), tag("//"))), not_line_ending))(input)
}

fn inline_comment(input: Input) -> IResult<Input, ()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(input)
}

fn comment(input: Input) -> IResult<Input, ()> {
    alt((line_comment, inline_comment))(input)
}

fn sp(input: Input) -> IResult<Input, ()> {
    value((), pair(space0, many0_count(pair(inline_comment, space0))))(input)
}

fn spc(input: Input) -> IResult<Input, ()> {
    value((), pair(space0, many0_count(pair(comment, space0))))(input)
}

fn ws(input: Input) -> IResult<Input, ()> {
    value(
        (),
        pair(multispace0, many0_count(pair(comment, multispace0))),
    )(input)
}

fn spanned<'a, F, O, T>(inner: F) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, T>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
    T: From<O> + Span,
{
    map(with_span(inner), |(value, span)| {
        let mut value = T::from(value);
        value.set_span(span);
        value
    })
}

fn with_span<'a, F, O>(
    mut parser: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, (O, Range<usize>)>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
{
    move |input: Input<'a>| {
        let start = input.location();

        match parser.parse(input) {
            Ok((rest, result)) => {
                let end = rest.location();
                Ok((rest, (result, start..end)))
            }
            Err(e) => Err(e),
        }
    }
}

fn span<'a, F, O>(mut parser: F) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Range<usize>>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
{
    move |input: Input<'a>| {
        let start = input.location();

        match parser.parse(input) {
            Ok((rest, _)) => {
                let end = rest.location();
                Ok((rest, start..end))
            }
            Err(e) => Err(e),
        }
    }
}

fn prefix_decor<'a, F, G, O1, O2, T>(
    prefix: F,
    inner: G,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, T>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O1>,
    G: FnMut(Input<'a>) -> IResult<Input<'a>, O2>,
    T: From<O2> + Decorate + Span,
{
    map(
        pair(span(prefix), with_span(inner)),
        |(prefix_span, (value, span))| {
            let mut value = T::from(value);
            value.decor_mut().set_prefix(prefix_span);

            value.set_span(span);
            value
        },
    )
}

fn suffix_decor<'a, F, G, O1, O2, T>(
    inner: F,
    suffix: G,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, T>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O1>,
    G: FnMut(Input<'a>) -> IResult<Input<'a>, O2>,
    T: From<O1> + Decorate + Span,
{
    map(
        pair(with_span(inner), span(suffix)),
        |((value, span), suffix_span)| {
            let mut value = T::from(value);
            value.decor_mut().set_suffix(suffix_span);

            value.set_span(span);
            value
        },
    )
}

fn decor<'a, F, G, H, O1, O2, O3, T>(
    prefix: F,
    inner: G,
    suffix: H,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, T>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O1>,
    G: FnMut(Input<'a>) -> IResult<Input<'a>, O2>,
    H: FnMut(Input<'a>) -> IResult<Input<'a>, O3>,
    T: From<O2> + Decorate + Span,
{
    map(
        tuple((span(prefix), with_span(inner), span(suffix))),
        |(prefix_span, (value, span), suffix_span)| {
            let mut value = T::from(value);
            let decor = value.decor_mut();
            decor.set_prefix(prefix_span);
            decor.set_suffix(suffix_span);
            value.set_span(span);
            value
        },
    )
}

fn hexescape<const N: usize>(input: Input) -> IResult<Input, char> {
    let parse_hex = verify(
        take_while_m_n(1, N, |c: u8| c.is_ascii_hexdigit()),
        |hex: &Input| hex.len() == N,
    );
    let parse_u32 = map_res(parse_hex, |hex: Input| {
        u32::from_str_radix(
            unsafe {
                from_utf8_unchecked(hex.input(), "`is_ascii_hexdigit` filters out non-ascii")
            },
            16,
        )
    });

    map_opt(parse_u32, std::char::from_u32)(input)
}

/// Parse an escaped character: `\n`, `\t`, `\r`, `\u00AC`, etc.
fn escaped_char(input: Input) -> IResult<Input, char> {
    preceded(
        char('\\'),
        alt((
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\\', char('\\')),
            value('"', char('"')),
            value('/', char('/')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            preceded(char('u'), hexescape::<4>),
            preceded(char('U'), hexescape::<8>),
        )),
    )(input)
}

/// Parse a non-empty block of text that doesn't include `\`,  `"` or non-escaped template
/// interpolation/directive start markers.
fn string_literal<'a>(input: Input<'a>) -> IResult<Input<'a>, &'a str> {
    literal(alt((recognize(one_of("\"\\")), tag("${"), tag("%{"))))(input)
}

fn literal<'a, F>(literal_end: F) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, &'a str>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, Input<'a>>,
{
    map_res(
        recognize(many1_count(alt((
            tag("$${"),
            tag("%%{"),
            anychar_except(literal_end),
        )))),
        |s| std::str::from_utf8(s.input()),
    )
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
        map(literal, StringFragment::Literal),
        map(escaped_char, StringFragment::EscapedChar),
    ))
}

fn string(input: Input) -> IResult<Input, String> {
    let build_string = fold_many0(
        string_fragment(string_literal),
        String::new,
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
            }
            string
        },
    );

    delimited(char('"'), build_string, char('"'))(input)
}

fn str_ident(input: Input) -> IResult<Input, &str> {
    context(
        "Identifier",
        map(
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
            )),
            |s: Input| unsafe {
                from_utf8_unchecked(
                    s.input(),
                    "`alpha1` and `alphanumeric1` filter out non-ascii",
                )
            },
        ),
    )(input)
}

fn ident(input: Input) -> IResult<Input, Identifier> {
    map(str_ident, Identifier::unchecked)(input)
}

fn exponent(input: Input) -> IResult<Input, Input> {
    recognize(tuple((one_of("eE"), opt(one_of("+-")), cut(digit1))))(input)
}

fn float(input: Input) -> IResult<Input, f64> {
    let fraction = preceded(char('.'), digit1);

    map_res(
        recognize(terminated(
            digit1,
            alt((terminated(fraction, opt(exponent)), exponent)),
        )),
        |s: Input| {
            f64::from_str(unsafe {
                from_utf8_unchecked(s.input(), "`digit1` and `exponent` filter out non-ascii")
            })
        },
    )(input)
}

fn integer(input: Input) -> IResult<Input, u64> {
    map_res(digit1, |s: Input| {
        u64::from_str(unsafe { from_utf8_unchecked(s.input(), "`digit1` filters out non-ascii") })
    })(input)
}

fn number(input: Input) -> IResult<Input, Number> {
    context(
        "Number",
        alt((map_opt(float, Number::from_f64), map(integer, Number::from))),
    )(input)
}

fn anychar_except<'a, F, T>(inner: F) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, Input<'a>>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, T>,
{
    recognize(preceded(not(inner), anychar))
}
