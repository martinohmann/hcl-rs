use crate::encode::{Encode, EncodeState};
use crate::raw_string::RawString;
use std::fmt::{self, Write};
use std::ops::{Deref, DerefMut, Range};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Decor {
    prefix: Option<RawString>,
    suffix: Option<RawString>,
}

impl Decor {
    pub fn new(prefix: impl Into<RawString>, suffix: impl Into<RawString>) -> Decor {
        Decor {
            prefix: Some(prefix.into()),
            suffix: Some(suffix.into()),
        }
    }

    pub fn from_prefix(prefix: impl Into<RawString>) -> Decor {
        Decor {
            prefix: Some(prefix.into()),
            suffix: None,
        }
    }

    pub fn from_suffix(suffix: impl Into<RawString>) -> Decor {
        Decor {
            prefix: None,
            suffix: Some(suffix.into()),
        }
    }

    pub fn set_prefix(&mut self, prefix: impl Into<RawString>) {
        self.prefix = Some(prefix.into());
    }

    pub fn set_suffix(&mut self, suffix: impl Into<RawString>) {
        self.suffix = Some(suffix.into());
    }

    pub fn prefix(&self) -> Option<&RawString> {
        self.prefix.as_ref()
    }

    pub fn suffix(&self) -> Option<&RawString> {
        self.suffix.as_ref()
    }

    pub fn replace(&mut self, other: Decor) -> Decor {
        std::mem::replace(self, other)
    }

    pub(crate) fn encode_prefix(&self, buf: &mut EncodeState, default: &str) -> fmt::Result {
        if let Some(prefix) = self.prefix() {
            prefix.encode_with_default(buf, default)
        } else {
            buf.write_str(default)
        }
    }

    pub(crate) fn encode_suffix(&self, buf: &mut EncodeState, default: &str) -> fmt::Result {
        if let Some(suffix) = self.suffix() {
            suffix.encode_with_default(buf, default)
        } else {
            buf.write_str(default)
        }
    }
}

impl Despan for Decor {
    fn despan(&mut self, input: &str) {
        if let Some(prefix) = &mut self.prefix {
            prefix.despan(input);
        }

        if let Some(suffix) = &mut self.suffix {
            suffix.despan(input);
        }
    }
}

impl<P, S> From<(P, S)> for Decor
where
    P: Into<RawString>,
    S: Into<RawString>,
{
    fn from((prefix, suffix): (P, S)) -> Self {
        Decor::new(prefix, suffix)
    }
}

pub trait Span {
    fn span(&self) -> Option<Range<usize>>;
}

pub(crate) trait SetSpan {
    fn set_span(&mut self, span: Range<usize>);

    fn spanned(mut self, span: Range<usize>) -> Self
    where
        Self: Sized,
    {
        self.set_span(span);
        self
    }
}

pub(crate) trait Despan {
    fn despan(&mut self, input: &str);
}

pub trait Decorate {
    fn decor(&self) -> &Decor;
    fn decor_mut(&mut self) -> &mut Decor;

    fn decorate(&mut self, decor: impl Into<Decor>) {
        *self.decor_mut() = decor.into();
    }

    fn decorated(mut self, decor: impl Into<Decor>) -> Self
    where
        Self: Sized,
    {
        self.decorate(decor);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    inner: T,
    span: Option<Range<usize>>,
}

impl<T> Spanned<T> {
    pub fn new(inner: T) -> Spanned<T> {
        Spanned { inner, span: None }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> AsMut<T> for Spanned<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> DerefMut for Spanned<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T> From<T> for Spanned<T> {
    fn from(value: T) -> Self {
        Spanned::new(value)
    }
}

impl<T> Span for Spanned<T> {
    fn span(&self) -> Option<Range<usize>> {
        self.span.clone()
    }
}

impl<T> SetSpan for Spanned<T> {
    fn set_span(&mut self, span: Range<usize>) {
        self.span = Some(span);
    }
}

impl<T> Despan for Spanned<T>
where
    T: Despan,
{
    fn despan(&mut self, input: &str) {
        self.inner.despan(input);
    }
}

impl<T> fmt::Display for Spanned<T>
where
    T: Encode,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode(&mut state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decorated<T> {
    inner: T,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl<T> Decorated<T> {
    pub fn new(inner: T) -> Decorated<T> {
        Decorated {
            inner,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> AsRef<T> for Decorated<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> AsMut<T> for Decorated<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Deref for Decorated<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> DerefMut for Decorated<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T> From<T> for Decorated<T> {
    fn from(value: T) -> Self {
        Decorated::new(value)
    }
}

impl<T> Decorate for Decorated<T> {
    fn decor(&self) -> &Decor {
        &self.decor
    }

    fn decor_mut(&mut self) -> &mut Decor {
        &mut self.decor
    }
}

impl<T> Span for Decorated<T> {
    fn span(&self) -> Option<Range<usize>> {
        self.span.clone()
    }
}

impl<T> SetSpan for Decorated<T> {
    fn set_span(&mut self, span: Range<usize>) {
        self.span = Some(span);
    }
}

impl<T> Despan for Decorated<T>
where
    T: Despan,
{
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.inner.despan(input);
    }
}

impl<T> fmt::Display for Decorated<T>
where
    T: Encode,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode(&mut state)
    }
}

impl<T> Decorate for Box<T>
where
    T: Decorate,
{
    fn decor(&self) -> &Decor {
        (**self).decor()
    }

    fn decor_mut(&mut self) -> &mut Decor {
        (**self).decor_mut()
    }
}

impl<T> Span for Box<T>
where
    T: Span,
{
    fn span(&self) -> Option<Range<usize>> {
        (**self).span()
    }
}

impl<T> SetSpan for Box<T>
where
    T: SetSpan,
{
    fn set_span(&mut self, span: Range<usize>) {
        (**self).set_span(span);
    }
}

impl<T> Despan for Box<T>
where
    T: Despan,
{
    fn despan(&mut self, input: &str) {
        (**self).despan(input);
    }
}
