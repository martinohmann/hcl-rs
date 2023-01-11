use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{multispace0, not_line_ending, space0};
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::multi::many0;
use nom::sequence::{pair, tuple};
use nom::IResult;

fn line_comment<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    recognize(tuple((alt((tag("#"), tag("//"))), not_line_ending)))(input)
}

fn block_comment<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    recognize(tuple((tag("/*"), take_until("*/"), tag("*/"))))(input)
}

fn comment<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    alt((line_comment, block_comment))(input)
}

pub fn sp<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    recognize(pair(space0, many0(pair(block_comment, space0))))(input)
}

pub fn ws<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    recognize(pair(multispace0, many0(pair(comment, multispace0))))(input)
}
