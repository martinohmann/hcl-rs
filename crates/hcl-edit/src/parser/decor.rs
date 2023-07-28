use super::prelude::*;
use crate::format::decor::{Decor, DecorFragment};
use winnow::{
    ascii::{not_line_ending, space1},
    combinator::{alt, repeat},
    token::{take_until0, take_while},
};

pub(crate) fn parse_multiline(input: &str) -> Option<Decor> {
    repeat::<_, _, Vec<_>, (), _>(
        0..,
        alt((
            space1.value(DecorFragment::Space),
            take_while(1.., is_line_break).map(DecorFragment::LineBreaks),
            (alt(("#", "//")), not_line_ending)
                .recognize()
                .map(DecorFragment::LineComment),
            ("/*", take_until0("*/"), "*/")
                .recognize()
                .map(DecorFragment::InlineComment),
        )),
    )
    .parse(input)
    .map(Into::into)
    .ok()
}

pub(crate) fn parse_inline(input: &str) -> Option<Decor> {
    repeat::<_, _, Vec<_>, (), _>(
        0..,
        alt((
            space1.value(DecorFragment::Space),
            ("/*", take_until0("*/"), "*/")
                .recognize()
                .map(DecorFragment::InlineComment),
        )),
    )
    .parse(input)
    .map(Into::into)
    .ok()
}

fn is_line_break(ch: char) -> bool {
    ch == '\n' || ch == '\r'
}
