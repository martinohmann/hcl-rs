use super::comment::{sp_comment0, ws_comment0};
use nom::{
    character::complete::{multispace0, space0},
    combinator::opt,
    error::ParseError,
    sequence::{delimited, preceded},
    IResult, Parser,
};

pub fn sp_delimited0<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    delimited(space0, inner, space0)
}

pub fn ws_delimited0<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn sp_comment_delimited0<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    delimited(sp_comment0, inner, sp_comment0)
}

pub fn ws_comment_delimited0<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    delimited(ws_comment0, inner, ws_comment0)
}

pub fn opt_sep<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, Option<O>, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    opt(preceded(ws_comment0, inner))
}
