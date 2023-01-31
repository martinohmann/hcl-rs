pub mod ast;
mod error;
mod expr;
mod span;
mod structure;
mod template;

pub use self::ast::*;
pub use self::error::{Error, ErrorKind, ParseResult};
use self::error::{IResult, InternalError};
use self::span::position;
use self::structure::body;
use crate::{Identifier, Number};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while_m_n},
    character::complete::{
        alpha1, alphanumeric1, anychar, char, digit1, line_ending, multispace0, not_line_ending,
        one_of, space0,
    },
    combinator::{all_consuming, cut, map, map_opt, map_res, not, opt, recognize, value},
    error::context,
    multi::{fold_many0, many0_count, many1_count},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Compare, CompareResult, Finish, InputLength, InputTake, Parser, Slice,
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
pub fn parse(input: &str) -> ParseResult<Node<Body>> {
    parse_to_end(input, body)
}

fn parse_to_end<'a, F, O>(input: &'a str, parser: F) -> ParseResult<O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    let input = Span::new(input);
    all_consuming(parser)
        .parse(input)
        .finish()
        .map(|(_, output)| output)
        .map_err(|err| Error::from_internal_error(input, err))
}

fn char_or_cut<'a>(ch: char) -> impl Fn(Span<'a>) -> IResult<Span<'a>, char> {
    move |input: Span<'a>| match input.chars().next().map(|t| t == ch) {
        Some(true) => Ok((input.slice(ch.len_utf8()..), ch)),
        _ => Err(nom::Err::Failure(InternalError::new(
            input,
            ErrorKind::Char(ch),
        ))),
    }
}

fn tag_or_cut<'a>(tag: &'a str) -> impl Fn(Span<'a>) -> IResult<Span<'a>, &'a str> {
    move |input: Span<'a>| {
        let tag_len = tag.input_len();
        match input.compare(tag) {
            CompareResult::Ok => {
                let (input, tag) = input.take_split(tag_len);
                Ok((input, *tag))
            }
            _ => Err(nom::Err::Failure(InternalError::new(
                input,
                ErrorKind::Tag(Span::new(tag)),
            ))),
        }
    }
}

fn line_comment(input: Span) -> IResult<Span, ()> {
    value(
        (),
        tuple((alt((tag("#"), tag("//"))), not_line_ending, line_ending)),
    )(input)
}

fn inline_comment(input: Span) -> IResult<Span, ()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(input)
}

fn comment(input: Span) -> IResult<Span, ()> {
    alt((line_comment, inline_comment))(input)
}

fn sp(input: Span) -> IResult<Span, ()> {
    value((), pair(space0, many0_count(pair(inline_comment, space0))))(input)
}

fn ws(input: Span) -> IResult<Span, ()> {
    value(
        (),
        pair(multispace0, many0_count(pair(comment, multispace0))),
    )(input)
}

fn spanned<'a, F, T>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Node<T>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, T>,
{
    map(with_span(inner), |(value, span)| Node::new(value, span))
}

fn with_span<'a, F, T>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, (T, Range<usize>)>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, T>,
{
    map(tuple((position, inner, position)), |(start, value, end)| {
        let span = start.location()..end.location();
        (value, span)
    })
}

fn span<'a, F, T>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Range<usize>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, T>,
{
    map(separated_pair(position, inner, position), |(start, end)| {
        start.location()..end.location()
    })
}

fn prefix_decorated<'a, F, G, O1, O2>(
    prefix: F,
    inner: G,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Node<O2>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O1>,
    G: FnMut(Span<'a>) -> IResult<Span<'a>, O2>,
{
    map(
        pair(span(prefix), with_span(inner)),
        |(prefix_span, (value, span))| {
            let decor = Decor::from_prefix(prefix_span);
            Node::new_with_decor(value, span, decor)
        },
    )
}

fn suffix_decorated<'a, F, G, O1, O2>(
    inner: F,
    suffix: G,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Node<O1>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O1>,
    G: FnMut(Span<'a>) -> IResult<Span<'a>, O2>,
{
    map(
        pair(with_span(inner), span(suffix)),
        |((value, span), suffix_span)| {
            let decor = Decor::from_suffix(suffix_span);
            Node::new_with_decor(value, span, decor)
        },
    )
}

fn decorated<'a, F, G, H, O1, O2, O3>(
    prefix: F,
    inner: G,
    suffix: H,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Node<O2>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O1>,
    G: FnMut(Span<'a>) -> IResult<Span<'a>, O2>,
    H: FnMut(Span<'a>) -> IResult<Span<'a>, O3>,
{
    map(
        tuple((span(prefix), with_span(inner), span(suffix))),
        |(prefix_span, (value, span), suffix_span)| {
            let decor = Decor::new(prefix_span, suffix_span);
            Node::new_with_decor(value, span, decor)
        },
    )
}

/// Parse a unicode sequence, of the form `uXXXX`, where XXXX is 1 to 6
/// hexadecimal numerals.
fn unicode(input: Span) -> IResult<Span, char> {
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'), parse_hex);
    let parse_u32 = map_res(parse_delimited_hex, move |hex: Span| {
        u32::from_str_radix(hex.input(), 16)
    });
    map_opt(parse_u32, std::char::from_u32)(input)
}

/// Parse an escaped character: `\n`, `\t`, `\r`, `\u00AC`, etc.
fn escaped_char(input: Span) -> IResult<Span, char> {
    preceded(
        char('\\'),
        alt((
            unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(input)
}

/// Parse a non-empty block of text that doesn't include `\`,  `"` or non-escaped template
/// interpolation/directive start markers.
fn string_literal(input: Span) -> IResult<Span, Span> {
    literal(alt((recognize(one_of("\"\\")), tag("${"), tag("%{"))))(input)
}

fn literal<'a, F>(literal_end: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Span<'a>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, Span<'a>>,
{
    recognize(many1_count(alt((
        tag("$${"),
        tag("%%{"),
        anything_except(literal_end),
    ))))
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
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, StringFragment<'a>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, Span<'a>>,
{
    alt((
        map(literal, |s| StringFragment::Literal(*s)),
        map(escaped_char, StringFragment::EscapedChar),
    ))
}

fn string(input: Span) -> IResult<Span, String> {
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

fn str_ident(input: Span) -> IResult<Span, &str> {
    context(
        "Identifier",
        map(
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
            )),
            |span: Span| *span,
        ),
    )(input)
}

fn ident(input: Span) -> IResult<Span, Identifier> {
    map(str_ident, Identifier::unchecked)(input)
}

fn exponent(input: Span) -> IResult<Span, Span> {
    recognize(tuple((one_of("eE"), opt(one_of("+-")), cut(digit1))))(input)
}

fn float(input: Span) -> IResult<Span, f64> {
    let fraction = preceded(char('.'), digit1);

    map_res(
        recognize(terminated(
            digit1,
            alt((terminated(fraction, opt(exponent)), exponent)),
        )),
        |s: Span| f64::from_str(s.input()),
    )(input)
}

fn integer(input: Span) -> IResult<Span, u64> {
    map_res(digit1, |s: Span| u64::from_str(s.input()))(input)
}

fn number(input: Span) -> IResult<Span, Number> {
    context(
        "Number",
        alt((map_opt(float, Number::from_f64), map(integer, Number::from))),
    )(input)
}

fn anything_except<'a, F>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Span<'a>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, Span<'a>>,
{
    recognize(preceded(not(inner), anychar))
}
