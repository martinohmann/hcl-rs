mod expr;
mod structure;
mod template;
#[cfg(test)]
mod tests;

use self::structure::body;
use self::template::template;
use crate::structure::Body;
use crate::template::Template;
use crate::{Identifier, Number, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_while_m_n},
    character::complete::{
        alpha1, alphanumeric1, char, multispace0, multispace1, not_line_ending, one_of, space0,
    },
    combinator::{all_consuming, map, map_opt, map_res, opt, recognize, value, verify},
    error::Error,
    multi::{fold_many0, many0, many0_count, many1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Finish, IResult, Parser,
};
use std::str::FromStr;

pub fn parse(input: &str) -> Result<Body> {
    all_consuming(body)
        .parse(input)
        .finish()
        .map(|(_, body)| body)
        .map_err(crate::Error::new)
}

pub fn parse_template(input: &str) -> Result<Template> {
    all_consuming(template)
        .parse(input)
        .finish()
        .map(|(_, template)| template)
        .map_err(crate::Error::new)
}

fn line_comment(input: &str) -> IResult<&str, &str> {
    recognize(tuple((alt((tag("#"), tag("//"))), not_line_ending)))(input)
}

fn block_comment(input: &str) -> IResult<&str, &str> {
    recognize(tuple((tag("/*"), take_until("*/"), tag("*/"))))(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    alt((line_comment, block_comment))(input)
}

fn sp(input: &str) -> IResult<&str, &str> {
    recognize(pair(space0, many0(pair(block_comment, space0))))(input)
}

fn ws(input: &str) -> IResult<&str, &str> {
    recognize(pair(multispace0, many0(pair(comment, multispace0))))(input)
}

fn sp_delimited<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Parser<&'a str, O, Error<&'a str>>,
{
    delimited(sp, inner, sp)
}

fn sp_terminated<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Parser<&'a str, O, Error<&'a str>>,
{
    terminated(inner, sp)
}

fn ws_delimited<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Parser<&'a str, O, Error<&'a str>>,
{
    delimited(ws, inner, ws)
}

fn ws_preceded<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Parser<&'a str, O, Error<&'a str>>,
{
    preceded(ws, inner)
}

fn ws_terminated<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Parser<&'a str, O, Error<&'a str>>,
{
    terminated(inner, ws)
}

fn opt_sep<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, Option<O>>
where
    F: Parser<&'a str, O, Error<&'a str>>,
{
    opt(ws_preceded(inner))
}

/// Parse a unicode sequence, of the form uXXXX, where XXXX is 1 to 6
/// hexadecimal numerals.
fn unicode(input: &str) -> IResult<&str, char> {
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'), parse_hex);
    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));
    map_opt(parse_u32, std::char::from_u32)(input)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
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

/// Parse a non-empty block of text that doesn't include \ or "
fn string_literal(input: &str) -> IResult<&str, &str> {
    let not_quote_slash = is_not("\"\\");
    verify(not_quote_slash, |s: &str| !s.is_empty())(input)
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

fn string_fragment(input: &str) -> IResult<&str, StringFragment<'_>> {
    alt((
        map(string_literal, StringFragment::Literal),
        map(escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, escaped_whitespace),
    ))(input)
}

fn string(input: &str) -> IResult<&str, String> {
    let build_string = fold_many0(string_fragment, String::new, |mut string, fragment| {
        match fragment {
            StringFragment::Literal(s) => string.push_str(s),
            StringFragment::EscapedChar(c) => string.push(c),
            StringFragment::EscapedWS => {}
        }
        string
    });

    delimited(char('"'), build_string, char('"'))(input)
}

fn str_ident(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
    ))(input)
}

fn ident(input: &str) -> IResult<&str, Identifier> {
    let not_a_keyword = |ident: &str| ident != "true" && ident != "false" && ident != "null";

    map(verify(str_ident, not_a_keyword), Identifier::unchecked)(input)
}

fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(many1(one_of("0123456789")))(input)
}

fn float(input: &str) -> IResult<&str, f64> {
    map_res(
        alt((
            // Case one: .42
            recognize(tuple((
                char('.'),
                decimal,
                opt(tuple((one_of("eE"), opt(one_of("+-")), decimal))),
            ))),
            // Case two: 42e42 and 42.42e42
            recognize(tuple((
                decimal,
                opt(preceded(char('.'), decimal)),
                one_of("eE"),
                opt(one_of("+-")),
                decimal,
            ))),
            // Case three: 42. and 42.42
            recognize(tuple((decimal, char('.'), opt(decimal)))),
        )),
        f64::from_str,
    )(input)
}

fn integer(input: &str) -> IResult<&str, u64> {
    map_res(decimal, u64::from_str)(input)
}

fn number(input: &str) -> IResult<&str, Number> {
    alt((map_opt(float, Number::from_f64), map(integer, Number::from)))(input)
}

fn boolean(input: &str) -> IResult<&str, bool> {
    let true_tag = value(true, tag("true"));
    let false_tag = value(false, tag("false"));
    alt((true_tag, false_tag))(input)
}

fn null(input: &str) -> IResult<&str, ()> {
    value((), tag("null"))(input)
}
