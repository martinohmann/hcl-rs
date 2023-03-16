use crate::encode::{Encode, EncodeState};
use crate::raw_string::RawString;
use crate::{private, Number};
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

    pub(crate) fn despan(&mut self, input: &str) {
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

impl<T> fmt::Display for Decorated<T>
where
    T: Encode,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode(&mut state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formatted<T> {
    inner: T,
    decor: Decor,
    repr: Option<RawString>,
    span: Option<Range<usize>>,
}

impl<T> Formatted<T>
where
    T: ValueRepr,
{
    pub fn new(inner: T) -> Formatted<T> {
        Formatted {
            inner,
            decor: Decor::default(),
            repr: None,
            span: None,
        }
    }

    pub fn repr(&self) -> Option<&RawString> {
        self.repr.as_ref()
    }

    pub(crate) fn set_repr(&mut self, repr: impl Into<RawString>) {
        self.repr = Some(repr.into());
    }

    pub(crate) fn with_repr(mut self, repr: impl Into<RawString>) -> Formatted<T> {
        self.set_repr(repr);
        self
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn format(&mut self) {
        self.set_repr(self.inner.to_repr())
    }
}

pub trait ValueRepr: private::Sealed {
    fn to_repr(&self) -> RawString;
}

impl private::Sealed for Number {}

impl ValueRepr for Number {
    fn to_repr(&self) -> RawString {
        RawString::from(self.to_string())
    }
}

impl<T> AsRef<T> for Formatted<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> AsMut<T> for Formatted<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Deref for Formatted<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> DerefMut for Formatted<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T> From<T> for Formatted<T>
where
    T: ValueRepr,
{
    fn from(value: T) -> Self {
        Formatted::new(value)
    }
}

impl<T> Decorate for Formatted<T> {
    fn decor(&self) -> &Decor {
        &self.decor
    }

    fn decor_mut(&mut self) -> &mut Decor {
        &mut self.decor
    }
}

impl<T> Span for Formatted<T> {
    fn span(&self) -> Option<Range<usize>> {
        self.span.clone()
    }
}

impl<T> SetSpan for Formatted<T> {
    fn set_span(&mut self, span: Range<usize>) {
        self.span = Some(span);
    }
}

impl<T> fmt::Display for Formatted<T>
where
    T: Encode,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode(&mut state)
    }
}
