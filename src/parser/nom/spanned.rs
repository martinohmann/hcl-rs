pub mod ast;
mod error;
mod expr;
mod structure;
mod template;

pub use self::ast::*;
pub use self::error::{Error, ErrorKind, ParseResult};
use self::error::{IResult, InternalError};
use self::structure::body;
use crate::structure::Body;
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
    sequence::{delimited, pair, preceded, terminated, tuple},
    Compare, CompareResult, Finish, InputLength, InputTake, Parser, Slice,
};
use nom_locate::position;
use std::str::FromStr;

fn spanned<'a, F, T>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Spanned<'a, T>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, T>,
{
    map(tuple((position, inner, position)), |(start, value, end)| {
        Spanned { start, value, end }
    })
}

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
pub fn parse(input: &str) -> ParseResult<Body> {
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

fn sp_delimited<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    delimited(sp, inner, sp)
}

fn sp_preceded<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    preceded(sp, inner)
}

fn sp_terminated<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    terminated(inner, sp)
}

fn ws_delimited<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    delimited(ws, inner, ws)
}

fn ws_preceded<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    preceded(ws, inner)
}

fn ws_terminated<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    terminated(inner, ws)
}

fn opt_sep<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Option<O>>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    opt(ws_preceded(inner))
}

/// Parse a unicode sequence, of the form `uXXXX`, where XXXX is 1 to 6
/// hexadecimal numerals.
fn unicode(input: Span) -> IResult<Span, char> {
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'), parse_hex);
    let parse_u32 = map_res(parse_delimited_hex, move |hex: Span| {
        u32::from_str_radix(hex.fragment(), 16)
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

fn str_ident(input: Span) -> IResult<Span, Span> {
    context(
        "Identifier",
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
        )),
    )(input)
}

fn ident(input: Span) -> IResult<Span, Identifier> {
    map(str_ident, |s: Span| Identifier::unchecked(*s))(input)
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
        |s: Span| f64::from_str(s.fragment()),
    )(input)
}

fn integer(input: Span) -> IResult<Span, u64> {
    map_res(digit1, |s: Span| u64::from_str(s.fragment()))(input)
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
