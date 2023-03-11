use super::{from_utf8_unchecked, IResult, Input};
use crate::repr::Decorated;
use crate::Ident;
use hcl_primitives::ident;
use winnow::{
    bytes::{one_of, take_while0},
    stream::AsChar,
    Parser,
};

pub(super) fn ident(input: Input) -> IResult<Input, Decorated<Ident>> {
    str_ident
        .map(|ident| Decorated::new(Ident::new_unchecked(ident)))
        .parse_next(input)
}

pub(super) fn str_ident(input: Input) -> IResult<Input, &str> {
    (one_of(is_id_start), take_while0(is_id_continue))
        .recognize()
        .map(|s: &[u8]| unsafe {
            from_utf8_unchecked(s, "`alpha1` and `alphanumeric1` filter out non-ascii")
        })
        .parse_next(input)
}

#[inline]
fn is_id_start(b: u8) -> bool {
    ident::is_id_start(b.as_char())
}

#[inline]
fn is_id_continue(b: u8) -> bool {
    ident::is_id_continue(b.as_char())
}
