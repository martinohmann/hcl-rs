use super::{
    error::ContextError,
    string::{ident, str_ident},
    Input,
};
use crate::{repr::Decorated, Ident};
use std::fmt;
use winnow::{combinator::cut_err, stream::AsChar, PResult, Parser};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum StrContext {
    Label(&'static str),
    Expected(StrContextValue),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum StrContextValue {
    CharLiteral(char),
    StringLiteral(&'static str),
    Description(&'static str),
}

impl fmt::Display for StrContextValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StrContextValue::CharLiteral('\n') => write!(f, "newline"),
            StrContextValue::CharLiteral(c) if c.is_ascii_control() => {
                write!(f, "`{}`", c.escape_debug())
            }
            StrContextValue::CharLiteral(c) => write!(f, "`{c}`"),
            StrContextValue::StringLiteral(l) => write!(f, "`{l}`"),
            StrContextValue::Description(d) => write!(f, "{d}"),
        }
    }
}

pub(super) fn cut_char<'a>(c: char) -> impl Parser<Input<'a>, char, ContextError<Input<'a>>> {
    cut_err(c)
        .map(AsChar::as_char)
        .context(StrContext::Expected(StrContextValue::CharLiteral(c)))
}

pub(super) fn cut_tag<'a>(
    tag: &'static str,
) -> impl Parser<Input<'a>, &'a [u8], ContextError<Input<'a>>> {
    cut_err(tag).context(StrContext::Expected(StrContextValue::StringLiteral(tag)))
}

pub(super) fn cut_ident<'a>(
    input: &mut Input<'a>,
) -> PResult<Decorated<Ident>, ContextError<Input<'a>>> {
    cut_err(ident)
        .context(StrContext::Expected(StrContextValue::Description(
            "identifier",
        )))
        .parse_next(input)
}

pub(super) fn cut_str_ident<'a>(
    input: &mut Input<'a>,
) -> PResult<&'a str, ContextError<Input<'a>>> {
    cut_err(str_ident)
        .context(StrContext::Expected(StrContextValue::Description(
            "identifier",
        )))
        .parse_next(input)
}
