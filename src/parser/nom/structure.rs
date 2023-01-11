use super::{
    combinators::{sp_delimited, sp_terminated, ws_preceded, ws_terminated},
    expr::expr,
    primitives::{ident, string},
};
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use nom::{
    branch::alt,
    character::complete::char,
    combinator::map,
    error::{context, ContextError, FromExternalError, ParseError},
    multi::many0,
    sequence::{delimited, separated_pair, tuple},
    IResult,
};
use std::num::ParseIntError;

fn attribute<'a, E>(input: &'a str) -> IResult<&'a str, Attribute, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "attribute",
        map(
            separated_pair(ident, sp_delimited(char('=')), expr),
            |(key, expr)| Attribute { key, expr },
        ),
    )(input)
}

fn block<'a, E>(input: &'a str) -> IResult<&'a str, Block, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    context(
        "block",
        map(
            tuple((
                sp_terminated(ident),
                many0(sp_terminated(block_label)),
                delimited(char('{'), body, char('}')),
            )),
            |(identifier, labels, body)| Block {
                identifier,
                labels,
                body,
            },
        ),
    )(input)
}

fn block_label<'a, E>(input: &'a str) -> IResult<&'a str, BlockLabel, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    alt((
        map(ident, BlockLabel::Identifier),
        map(string, BlockLabel::String),
    ))(input)
}

fn structure<'a, E>(input: &'a str) -> IResult<&'a str, Structure, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    alt((
        map(attribute, Structure::Attribute),
        map(block, Structure::Block),
    ))(input)
}

pub fn body<'a, E>(input: &'a str) -> IResult<&'a str, Body, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    ws_preceded(map(many0(ws_terminated(structure)), Body::from))(input)
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_attribute() {
        assert_eq!(
            body::<VerboseError<&str>>("foo = \"bar\"\nbar = 2\n\n"),
            Ok((
                "",
                Body::builder()
                    .add_attribute(Attribute::new("foo", "bar"))
                    .add_attribute(Attribute::new("bar", 2u64))
                    .build()
            )),
        );
    }
}
