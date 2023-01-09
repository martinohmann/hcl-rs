mod combinators;
mod comment;
mod expr;
mod string;
mod structure;
mod template;
#[cfg(test)]
mod tests;

use self::structure::body;
use crate::structure::Body;
use crate::{Error, Identifier, Result};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1};
use nom::combinator::{all_consuming, map, recognize, verify};
use nom::error::{convert_error, ParseError, VerboseError};
use nom::multi::many0_count;
use nom::sequence::pair;
use nom::{Finish, IResult, Parser};

fn ident<'a, E>(input: &'a str) -> IResult<&'a str, Identifier, E>
where
    E: ParseError<&'a str>,
{
    map(
        verify(str_ident, |ident: &str| {
            ident != "true" && ident != "false" && ident != "null"
        }),
        Identifier::unchecked,
    )(input)
}

fn str_ident<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
    ))(input)
}

pub fn parse<'a>(input: &'a str) -> Result<Body> {
    all_consuming(|input| body::<VerboseError<&'a str>>(input))
        .parse(input)
        .finish()
        .map(|(_, body)| body)
        .map_err(|err| Error::new(convert_error(input, err)))
}
