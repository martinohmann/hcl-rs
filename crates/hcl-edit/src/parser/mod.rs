//! An HCL parser which keeps track of whitespace, comments and span information.

mod context;
mod error;
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

pub use self::error::{Error, Location};
use self::{error::ParseError, expr::expr, structure::body, template::template};
use crate::{expr::Expression, structure::Body, template::Template};
use winnow::{stream::Located, Parser};

type Input<'a> = Located<&'a [u8]>;

type IResult<I, O, E = ParseError<I>> = winnow::IResult<I, O, E>;

/// Parse an input into a [`Body`](crate::structure::Body).
pub fn parse_body(input: &str) -> Result<Body, Error> {
    let mut body = parse_complete(input, body)?;
    body.despan(input);
    Ok(body)
}

/// Parse an input into an [`Expression`](crate::expr::Expression).
pub fn parse_expr(input: &str) -> Result<Expression, Error> {
    let mut expr = parse_complete(input, expr)?;
    expr.despan(input);
    Ok(expr)
}

/// Parse an input into a [`Template`](crate::template::Template).
pub fn parse_template(input: &str) -> Result<Template, Error> {
    let mut template = parse_complete(input, template)?;
    template.despan(input);
    Ok(template)
}

fn parse_complete<'a, P, O>(input: &'a str, mut parser: P) -> Result<O, Error>
where
    P: Parser<Input<'a>, O, ParseError<Input<'a>>>,
{
    let input = Input::new(input.as_bytes());

    parser
        .parse(input)
        .map_err(|err| Error::from_parse_error(input, err))
}
