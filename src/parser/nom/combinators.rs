use super::comment::{sp, ws};
use nom::{
    combinator::opt,
    error::ParseError,
    sequence::{delimited, preceded, terminated},
    IResult, Parser,
};

pub fn sp_delimited<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    delimited(sp, inner, sp)
}

pub fn sp_terminated<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    terminated(inner, sp)
}

pub fn ws_delimited<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    delimited(ws, inner, ws)
}

pub fn ws_preceded<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    preceded(ws, inner)
}

pub fn ws_terminated<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    terminated(inner, ws)
}

pub fn opt_sep<'a, F, O, E>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, Option<O>, E>
where
    F: Parser<&'a str, O, E> + 'a,
    E: ParseError<&'a str>,
{
    opt(ws_preceded(inner))
}
