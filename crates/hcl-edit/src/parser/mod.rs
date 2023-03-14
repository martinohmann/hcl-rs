mod context;
pub mod error;
mod expr;
mod number;
mod repr;
mod state;
mod string;
mod structure;
mod template;
#[cfg(test)]
mod tests;
mod trivia;

use self::{
    error::{Error, ParseError, ParseResult},
    expr::expr,
    structure::body,
    template::template,
};
use crate::{expr::Expression, structure::Body, template::Template};
use winnow::{prelude::*, stream::Located, Parser};

type Input<'a> = Located<&'a [u8]>;

type IResult<I, O, E = ParseError<I>> = winnow::IResult<I, O, E>;

pub fn parse_body(input: &str) -> ParseResult<Body> {
    let mut body = parse_complete(input, body)?;
    body.despan(input);
    Ok(body)
}

pub fn parse_expr(input: &str) -> ParseResult<Expression> {
    let mut expr = parse_complete(input, expr)?;
    expr.despan(input);
    Ok(expr)
}

pub fn parse_template(input: &str) -> ParseResult<Template> {
    let mut template = parse_complete(input, template)?;
    template.despan(input);
    Ok(template)
}

fn parse_complete<'a, P, O>(input: &'a str, mut parser: P) -> ParseResult<O>
where
    P: Parser<Input<'a>, O, ParseError<Input<'a>>>,
{
    let input = Input::new(input.as_bytes());

    parser
        .parse_next(input)
        .finish()
        .map_err(|err| Error::from_parse_error(input, err))
}
