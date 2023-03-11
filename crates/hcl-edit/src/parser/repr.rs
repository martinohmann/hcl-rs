use super::error::InternalError;
use super::string::raw;
use super::Input;
use crate::repr::{Decorate, SetSpan};
use winnow::Parser;

pub(super) fn spanned<'a, F, O>(inner: F) -> impl Parser<Input<'a>, O, InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, O, InternalError<Input<'a>>>,
    O: SetSpan,
{
    inner.with_span().map(|(mut value, span)| {
        value.set_span(span);
        value
    })
}

pub(super) fn prefix_decor<'a, F, G, O1, O2>(
    prefix: F,
    inner: G,
) -> impl Parser<Input<'a>, O2, InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, O1, InternalError<Input<'a>>>,
    G: Parser<Input<'a>, O2, InternalError<Input<'a>>>,
    O2: Decorate + SetSpan,
{
    (raw(prefix), inner.with_span()).map(|(prefix, (mut value, span))| {
        value.decor_mut().set_prefix(prefix);
        value.set_span(span);
        value
    })
}

pub(super) fn suffix_decor<'a, F, G, O1, O2>(
    inner: F,
    suffix: G,
) -> impl Parser<Input<'a>, O1, InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, O1, InternalError<Input<'a>>>,
    G: Parser<Input<'a>, O2, InternalError<Input<'a>>>,
    O1: Decorate + SetSpan,
{
    (inner.with_span(), raw(suffix)).map(|((mut value, span), suffix)| {
        value.decor_mut().set_suffix(suffix);
        value.set_span(span);
        value
    })
}

pub(super) fn decor<'a, F, G, H, O1, O2, O3>(
    prefix: F,
    inner: G,
    suffix: H,
) -> impl Parser<Input<'a>, O2, InternalError<Input<'a>>>
where
    F: Parser<Input<'a>, O1, InternalError<Input<'a>>>,
    G: Parser<Input<'a>, O2, InternalError<Input<'a>>>,
    H: Parser<Input<'a>, O3, InternalError<Input<'a>>>,
    O2: Decorate + SetSpan,
{
    (raw(prefix), inner.with_span(), raw(suffix)).map(|(prefix, (mut value, span), suffix)| {
        let decor = value.decor_mut();
        decor.set_prefix(prefix);
        decor.set_suffix(suffix);
        value.set_span(span);
        value
    })
}
