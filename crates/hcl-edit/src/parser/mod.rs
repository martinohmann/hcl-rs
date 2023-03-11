#![allow(missing_docs)]

mod context;
mod error;
mod expr;
mod ident;
mod number;
mod repr;
mod string;
mod structure;
mod template;
#[cfg(test)]
mod tests;
mod trivia;

pub use self::error::{Error, ParseResult};

use self::{error::InternalError, expr::expr, structure::body, template::template};
use crate::{expr::Expression, repr::Despan, structure::Body, template::Template};
use winnow::{prelude::*, stream::Located, Parser};

pub(crate) type Input<'a> = Located<&'a [u8]>;

pub(crate) type IResult<I, O, E = InternalError<I>> = winnow::IResult<I, O, E>;

pub fn parse_body(input: &str) -> ParseResult<Body> {
    let mut body = parse_to_end(input, body)?;
    body.despan(input);
    Ok(body)
}

pub fn parse_expr(input: &str) -> ParseResult<Expression> {
    let mut expr = parse_to_end(input, expr)?;
    expr.despan(input);
    Ok(expr)
}

pub fn parse_template(input: &str) -> ParseResult<Template> {
    let mut template = parse_to_end(input, template)?;
    template.despan(input);
    Ok(template)
}

fn parse_to_end<'a, F, O>(input: &'a str, mut parser: F) -> ParseResult<O>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, O>,
{
    let input = Input::new(input.as_bytes());
    parser
        .parse_next(input)
        .finish()
        .map_err(|err| Error::from_internal_error(input, err))
}

unsafe fn from_utf8_unchecked<'b>(bytes: &'b [u8], safety_justification: &'static str) -> &'b str {
    if cfg!(debug_assertions) {
        std::str::from_utf8(bytes).expect(safety_justification)
    } else {
        std::str::from_utf8_unchecked(bytes)
    }
}
