use super::prelude::*;

use crate::{Decorate, RawString, SetSpan};

pub(super) fn spanned<'a, F, O>(inner: F) -> impl ModalParser<Input<'a>, O, ContextError>
where
    F: ModalParser<Input<'a>, O, ContextError>,
    O: SetSpan,
{
    inner.with_span().map(|(mut value, span)| {
        value.set_span(span);
        value
    })
}

pub(super) fn decorated<'a, F, G, H, O1, O2, O3>(
    prefix: F,
    inner: G,
    suffix: H,
) -> impl ModalParser<Input<'a>, O2, ContextError>
where
    F: ModalParser<Input<'a>, O1, ContextError>,
    G: ModalParser<Input<'a>, O2, ContextError>,
    H: ModalParser<Input<'a>, O3, ContextError>,
    O2: Decorate + SetSpan,
{
    (prefix.span(), spanned(inner), suffix.span()).map(|(prefix, mut value, suffix)| {
        let decor = value.decor_mut();
        decor.set_prefix(RawString::from_span(prefix));
        decor.set_suffix(RawString::from_span(suffix));
        value
    })
}

pub(super) fn prefix_decorated<'a, F, G, O1, O2>(
    prefix: F,
    inner: G,
) -> impl ModalParser<Input<'a>, O2, ContextError>
where
    F: ModalParser<Input<'a>, O1, ContextError>,
    G: ModalParser<Input<'a>, O2, ContextError>,
    O2: Decorate + SetSpan,
{
    (prefix.span(), spanned(inner)).map(|(prefix, mut value)| {
        value.decor_mut().set_prefix(RawString::from_span(prefix));
        value
    })
}

pub(super) fn suffix_decorated<'a, F, G, O1, O2>(
    inner: F,
    suffix: G,
) -> impl ModalParser<Input<'a>, O1, ContextError>
where
    F: ModalParser<Input<'a>, O1, ContextError>,
    G: ModalParser<Input<'a>, O2, ContextError>,
    O1: Decorate + SetSpan,
{
    (spanned(inner), suffix.span()).map(|(mut value, suffix)| {
        value.decor_mut().set_suffix(RawString::from_span(suffix));
        value
    })
}
