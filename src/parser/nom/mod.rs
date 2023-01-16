mod error;
mod expr;
mod structure;
mod template;
#[cfg(test)]
mod tests;

use self::error::IResult;
pub use self::error::{Error, ErrorKind, Location, ParseResult};
use self::structure::body;
use self::template::template;
use crate::structure::Body;
use crate::template::Template;
use crate::{Identifier, Number};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while_m_n},
    character::complete::{
        alpha1, alphanumeric1, anychar, char, digit1, line_ending, multispace0, multispace1,
        not_line_ending, one_of, space0,
    },
    combinator::{all_consuming, cut, map, map_opt, map_res, not, opt, recognize, value},
    error::context,
    multi::{fold_many0, many0_count, many1_count},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Finish, Parser,
};
use std::str::FromStr;

pub fn parse(input: &str) -> ParseResult<Body> {
    parse_to_end(input, body)
}

pub fn parse_template(input: &str) -> ParseResult<Template> {
    parse_to_end(input, template)
}

fn parse_to_end<'a, F, O>(input: &'a str, parser: F) -> ParseResult<O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    all_consuming(parser)
        .parse(input)
        .finish()
        .map(|(_, output)| output)
        .map_err(|err| Error::from_internal_error(input, err))
}

fn line_comment(input: &str) -> IResult<&str, ()> {
    value(
        (),
        tuple((alt((tag("#"), tag("//"))), not_line_ending, line_ending)),
    )(input)
}

fn inline_comment(input: &str) -> IResult<&str, ()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(input)
}

fn comment(input: &str) -> IResult<&str, ()> {
    alt((line_comment, inline_comment))(input)
}

fn sp(input: &str) -> IResult<&str, ()> {
    value((), pair(space0, many0_count(pair(inline_comment, space0))))(input)
}

fn ws(input: &str) -> IResult<&str, ()> {
    value(
        (),
        pair(multispace0, many0_count(pair(comment, multispace0))),
    )(input)
}

fn sp_delimited<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(sp, inner, sp)
}

fn sp_preceded<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    preceded(sp, inner)
}

fn sp_terminated<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    terminated(inner, sp)
}

fn ws_delimited<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(ws, inner, ws)
}

fn ws_preceded<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    preceded(ws, inner)
}

fn ws_terminated<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    terminated(inner, ws)
}

fn opt_sep<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, Option<O>>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    opt(ws_preceded(inner))
}

/// Parse a unicode sequence, of the form `uXXXX`, where XXXX is 1 to 6
/// hexadecimal numerals.
fn unicode(input: &str) -> IResult<&str, char> {
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'), parse_hex);
    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));
    map_opt(parse_u32, std::char::from_u32)(input)
}

/// Parse an escaped character: `\n`, `\t`, `\r`, `\u00AC`, etc.
fn escaped_char(input: &str) -> IResult<&str, char> {
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

/// Parse a backslash, followed by any amount of whitespace. This is used to discard any escaped
/// whitespace.
fn escaped_whitespace(input: &str) -> IResult<&str, &str> {
    preceded(char('\\'), multispace1)(input)
}

/// Parse a non-empty block of text that doesn't include `\`,  `"` or non-escaped template
/// interpolation/directive start markers.
fn string_literal(input: &str) -> IResult<&str, &str> {
    recognize(many1_count(alt((
        tag("$${"),
        tag("%%{"),
        anything_except(alt((tag("\""), tag("\\"), tag("${"), tag("%{")))),
    ))))(input)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

fn string_fragment<'a, F>(
    literal_parser: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, StringFragment<'a>>
where
    F: FnMut(&'a str) -> IResult<&'a str, &'a str>,
{
    alt((
        map(literal_parser, StringFragment::Literal),
        map(escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, escaped_whitespace),
    ))
}

fn string(input: &str) -> IResult<&str, String> {
    let build_string = fold_many0(
        string_fragment(string_literal),
        String::new,
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        },
    );

    delimited(char('"'), build_string, char('"'))(input)
}

fn str_ident(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
    ))(input)
}

fn ident(input: &str) -> IResult<&str, Identifier> {
    context("Identifier", map(str_ident, Identifier::unchecked))(input)
}

fn exponent(input: &str) -> IResult<&str, &str> {
    recognize(tuple((one_of("eE"), opt(one_of("+-")), cut(digit1))))(input)
}

fn float(input: &str) -> IResult<&str, f64> {
    let fraction = preceded(char('.'), digit1);

    map_res(
        recognize(terminated(
            digit1,
            alt((terminated(fraction, opt(exponent)), exponent)),
        )),
        f64::from_str,
    )(input)
}

fn integer(input: &str) -> IResult<&str, u64> {
    map_res(digit1, u64::from_str)(input)
}

fn number(input: &str) -> IResult<&str, Number> {
    context(
        "Number",
        alt((map_opt(float, Number::from_f64), map(integer, Number::from))),
    )(input)
}

fn anything_except<'a, F>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str>
where
    F: FnMut(&'a str) -> IResult<&'a str, &'a str>,
{
    recognize(preceded(not(inner), anychar))
}
