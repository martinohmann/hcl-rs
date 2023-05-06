use super::{
    error::ParseError,
    string::{ident, str_ident},
    IResult, Input,
};
use crate::{repr::Decorated, Ident};
use std::fmt;
use winnow::{combinator::cut_err, stream::AsChar, Parser};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Context {
    Expression(&'static str),
    Expected(Expected),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Expected {
    Char(char),
    Literal(&'static str),
    Description(&'static str),
}

impl fmt::Display for Expected {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expected::Char('\n') => write!(f, "newline"),
            Expected::Char(c) if c.is_ascii_control() => {
                write!(f, "`{}`", c.escape_debug())
            }
            Expected::Char(c) => write!(f, "`{c}`"),
            Expected::Literal(l) => write!(f, "`{l}`"),
            Expected::Description(d) => write!(f, "{d}"),
        }
    }
}

pub(super) fn cut_char<'a>(c: char) -> impl Parser<Input<'a>, char, ParseError<Input<'a>>> {
    cut_err(c)
        .map(AsChar::as_char)
        .context(Context::Expected(Expected::Char(c)))
}

pub(super) fn cut_tag<'a>(
    tag: &'static str,
) -> impl Parser<Input<'a>, &'a [u8], ParseError<Input<'a>>> {
    cut_err(tag).context(Context::Expected(Expected::Literal(tag)))
}

pub(super) fn cut_ident(input: Input) -> IResult<Input, Decorated<Ident>> {
    cut_err(ident)
        .context(Context::Expected(Expected::Description("identifier")))
        .parse_next(input)
}

pub(super) fn cut_str_ident(input: Input) -> IResult<Input, &str> {
    cut_err(str_ident)
        .context(Context::Expected(Expected::Description("identifier")))
        .parse_next(input)
}
