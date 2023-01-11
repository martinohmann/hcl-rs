mod expr;
mod primitives;
mod structure;
mod template;
#[cfg(test)]
mod tests;

use self::structure::body;
use self::template::template;
use crate::structure::Body;
use crate::template::Template;
use crate::Result;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{multispace0, not_line_ending, space0},
    combinator::{all_consuming, opt, recognize},
    error::Error,
    multi::many0,
    sequence::{delimited, pair, preceded, terminated, tuple},
    Finish, IResult, Parser,
};

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
