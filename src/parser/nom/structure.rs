use super::{
    expr::expr,
    primitives::{ident, string},
    sp_delimited, sp_terminated, ws_preceded, ws_terminated,
};
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use nom::{
    branch::alt,
    character::complete::char,
    combinator::map,
    error::context,
    multi::many0,
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

fn attribute(input: &str) -> IResult<&str, Attribute> {
    context(
        "attribute",
        map(
            separated_pair(ident, sp_delimited(char('=')), expr),
            |(key, expr)| Attribute { key, expr },
        ),
    )(input)
}

fn block(input: &str) -> IResult<&str, Block> {
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

fn block_label(input: &str) -> IResult<&str, BlockLabel> {
    alt((
        map(ident, BlockLabel::Identifier),
        map(string, BlockLabel::String),
    ))(input)
}

fn structure(input: &str) -> IResult<&str, Structure> {
    alt((
        map(attribute, Structure::Attribute),
        map(block, Structure::Block),
    ))(input)
}

pub fn body(input: &str) -> IResult<&str, Body> {
    ws_preceded(map(many0(ws_terminated(structure)), Body::from))(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_attribute() {
        assert_eq!(
            body("foo = \"bar\"\nbar = 2\n\n"),
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
