use super::{
    combinators::sp_comment_delimited0,
    comment::{sp_comment0, ws_comment0},
    expr::expr,
    ident,
    string::string,
};
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::map,
    error::{context, ContextError, FromExternalError, ParseError},
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
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
            separated_pair(ident, sp_comment_delimited0(char('=')), expr),
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
            pair(
                terminated(ident, sp_comment0),
                pair(many0(terminated(block_label, sp_comment0)), block_body),
            ),
            |(identifier, (labels, body))| Block {
                identifier,
                labels,
                body,
            },
        ),
    )(input)
}

fn block_body<'a, E>(input: &'a str) -> IResult<&'a str, Body, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + 'a,
{
    delimited(tag("{"), body, tag("}"))(input)
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
    preceded(
        ws_comment0,
        map(many0(terminated(structure, ws_comment0)), Into::into),
    )(input)
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
