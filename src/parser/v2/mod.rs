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
use nom::combinator::{all_consuming, map, recognize};
use nom::error::{ParseError, VerboseError};
use nom::multi::many0_count;
use nom::sequence::pair;
use nom::{IResult, Parser};

fn ident<'a, E>(input: &'a str) -> IResult<&'a str, Identifier, E>
where
    E: ParseError<&'a str>,
{
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
        )),
        Identifier::unchecked,
    )(input)
}

pub fn parse<'a>(input: &'a str) -> Result<Body> {
    all_consuming(|input| body::<VerboseError<&'a str>>(input))
        .parse(input)
        .map(|(_, body)| body)
        .map_err(Error::new)
}
