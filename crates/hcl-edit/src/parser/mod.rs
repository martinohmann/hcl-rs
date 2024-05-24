//! An HCL parser which keeps track of whitespace, comments and span information.

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
use self::expr::expr;
use self::structure::body;
use self::template::template;
use crate::expr::Expression;
use crate::structure::Body;
use crate::template::Template;

mod prelude {
    pub(super) use winnow::error::{ContextError, StrContext, StrContextValue};
    pub(super) use winnow::stream::Stream;
    pub(super) use winnow::{dispatch, PResult, Parser};

    pub(super) type Input<'a> = winnow::stream::Located<&'a str>;
}

use self::prelude::*;

/// Parse an input into a [`Body`].
///
/// # Errors
///
/// Returns an error if the input does not resemble a valid HCL body.
pub fn parse_body(input: &str) -> Result<Body, Error> {
    let mut body = parse_complete(input, body)?;
    body.despan(input);
    Ok(body)
}

/// Parse an input into an [`Expression`].
///
/// # Errors
///
/// Returns an error if the input does not resemble a valid HCL expression.
pub fn parse_expr(input: &str) -> Result<Expression, Error> {
    let mut expr = parse_complete(input, expr)?;
    expr.despan(input);
    Ok(expr)
}

/// Parse an input into a [`Template`].
///
/// # Errors
///
/// Returns an error if the input does not resemble a valid HCL template.
pub fn parse_template(input: &str) -> Result<Template, Error> {
    let mut template = parse_complete(input, template)?;
    template.despan(input);
    Ok(template)
}

fn parse_complete<'a, P, O>(input: &'a str, mut parser: P) -> Result<O, Error>
where
    P: Parser<Input<'a>, O, ContextError>,
{
    let input = Input::new(input);

    parser
        .parse(input)
        .map_err(|err| Error::from_parse_error(&err))
}
