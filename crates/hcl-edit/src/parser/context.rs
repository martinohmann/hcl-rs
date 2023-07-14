use super::{
    string::{ident, str_ident},
    Input,
};
use crate::{repr::Decorated, Ident};
use winnow::{
    combinator::cut_err,
    error::{ContextError, StrContext, StrContextValue},
    stream::AsChar,
    PResult, Parser,
};

pub(super) fn cut_char<'a>(c: char) -> impl Parser<Input<'a>, char, ContextError> {
    cut_err(c)
        .map(AsChar::as_char)
        .context(StrContext::Expected(StrContextValue::CharLiteral(c)))
}

pub(super) fn cut_tag<'a>(tag: &'static str) -> impl Parser<Input<'a>, &'a [u8], ContextError> {
    cut_err(tag).context(StrContext::Expected(StrContextValue::StringLiteral(tag)))
}

pub(super) fn cut_ident<'a>(input: &mut Input<'a>) -> PResult<Decorated<Ident>> {
    cut_err(ident)
        .context(StrContext::Expected(StrContextValue::Description(
            "identifier",
        )))
        .parse_next(input)
}

pub(super) fn cut_str_ident<'a>(input: &mut Input<'a>) -> PResult<&'a str> {
    cut_err(str_ident)
        .context(StrContext::Expected(StrContextValue::Description(
            "identifier",
        )))
        .parse_next(input)
}
