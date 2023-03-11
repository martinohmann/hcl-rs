#![allow(missing_docs)]

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

use self::error::{Context, Expected, InternalError};
pub use self::error::{Error, ParseResult};
use self::expr::expr;
use self::ident::ident;
use self::structure::body;
use self::template::template;
use crate::expr::Expression;
use crate::repr::{Decorated, Despan};
use crate::structure::Body;
use crate::template::Template;
use crate::Ident;
use winnow::{
    bytes::{any, one_of},
    combinator::{cut_err, not},
    prelude::*,
    sequence::preceded,
    stream::{AsChar, Located},
    Parser,
};

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

fn cut_char<'a>(c: char) -> impl Parser<Input<'a>, char, InternalError<Input<'a>>> {
    cut_err(one_of(c))
        .map(AsChar::as_char)
        .context(Context::Expected(Expected::Char(c)))
}

fn cut_tag<'a>(t: &'static str) -> impl Parser<Input<'a>, &'a [u8], InternalError<Input<'a>>> {
    cut_err(t).context(Context::Expected(Expected::Literal(t)))
}

fn cut_ident(input: Input) -> IResult<Input, Decorated<Ident>> {
    cut_err(ident)
        .context(Context::Expected(Expected::Description("identifier")))
        .parse_next(input)
}

fn any_except<'a, F, T>(inner: F) -> impl Parser<Input<'a>, &'a [u8], InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, T, InternalError<Input<'a>>>,
{
    preceded(not(inner), any).recognize()
}

#[inline]
fn void<'a, P>(inner: P) -> impl Parser<Input<'a>, (), InternalError<Input<'a>>>
where
    P: Parser<Input<'a>, (), InternalError<Input<'a>>>,
{
    inner
}
