//! Representations of values within a HCL document.

use crate::encode::{Encode, EncodeState};
use crate::format::Formatter;
use crate::raw_string::RawString;
use std::fmt::{self, Write};
use std::ops::{Deref, DerefMut, Range};

/// Represents the whitespace and comments before (the "prefix") or after (the "suffix") a HCL value.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Decor {
    pub(crate) prefix: Option<RawString>,
    pub(crate) suffix: Option<RawString>,
}

impl Decor {
    /// Creates a new `Decor` for a prefix and a suffix.
    pub fn new(prefix: impl Into<RawString>, suffix: impl Into<RawString>) -> Decor {
        Decor {
            prefix: Some(prefix.into()),
            suffix: Some(suffix.into()),
        }
    }

    /// Sets the decor prefix.
    pub fn set_prefix(&mut self, prefix: impl Into<RawString>) {
        self.prefix = Some(prefix.into());
    }

    /// Sets the decor suffix.
    pub fn set_suffix(&mut self, suffix: impl Into<RawString>) {
        self.suffix = Some(suffix.into());
    }

    /// Returns a reference to the decor prefix, if one is present, `None` otherwise.
    pub fn prefix(&self) -> Option<&RawString> {
        self.prefix.as_ref()
    }

    /// Returns a reference to the decor suffix, if one is present, `None` otherwise.
    pub fn suffix(&self) -> Option<&RawString> {
        self.suffix.as_ref()
    }

    pub(crate) fn is_multiline(&self) -> bool {
        self.prefix.as_ref().map_or(false, RawString::is_multiline)
            || self.suffix.as_ref().map_or(false, RawString::is_multiline)
    }

    /// Clears the decor prefix and suffix.
    pub fn clear(&mut self) {
        self.prefix = None;
        self.suffix = None;
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

/// A trait for objects which carry span information.
pub trait Span {
    /// Obtains the span information. This only returns `Some` if the value was emitted by the
    /// parser.
    ///
    /// The returned range represents a zero-based start and end byte offset in the input from
    /// which this object was parsed.
    fn span(&self) -> Option<Range<usize>>;
}

pub(crate) trait SetSpan {
    fn set_span(&mut self, span: Range<usize>);
}

/// A trait for objects which can be decorated with whitespace and comments.
pub trait Decorate {
    /// Returns a reference to the object's [`Decor`].
    fn decor(&self) -> &Decor;

    /// Returns a mutable reference to the object's [`Decor`].
    fn decor_mut(&mut self) -> &mut Decor;

    /// Decorate the object with `decor` in-place.
    fn decorate(&mut self, decor: impl Into<Decor>) {
        *self.decor_mut() = decor.into();
    }

    /// Decorate the object with `decor` and return the modified value.
    fn decorated(mut self, decor: impl Into<Decor>) -> Self
    where
        Self: Sized,
    {
        self.decorate(decor);
        self
    }
}

/// A trait for objects which can be formatted.
pub trait Format {
    /// Formats an object.
    fn format(&mut self, formatter: Formatter);

    /// Applies the default format to an object.
    fn default_format(&mut self) {
        let formatter = Formatter::default();
        self.format(formatter);
    }

    /// Formats an object and returns the modified value.
    fn formatted(mut self, formatter: Formatter) -> Self
    where
        Self: Sized,
    {
        self.format(formatter);
        self
    }

    /// Applies the default format to an object and returns the modified value.
    fn default_formatted(mut self) -> Self
    where
        Self: Sized,
    {
        self.default_format();
        self
    }
}

/// A wrapper type for attaching span information to a value.
#[derive(Debug, Clone, Eq)]
pub struct Spanned<T> {
    value: T,
    span: Option<Range<usize>>,
}

impl<T> Spanned<T> {
    /// Creates a new `Spanned<T>` from a `T`.
    pub fn new(value: T) -> Spanned<T> {
        Spanned { value, span: None }
    }

    /// Consumes the `Spanned<T>` and returns the wrapped value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Returns a reference to the wrapped value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the wrapped value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> PartialEq for Spanned<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> AsRef<T> for Spanned<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.value()
    }
}

impl<T> AsMut<T> for Spanned<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.value_mut()
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

/// A wrapper type for attaching a [`Decor`] and span information to a value.
#[derive(Debug, Clone, Eq)]
pub struct Decorated<T> {
    value: T,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl<T> Decorated<T> {
    /// Creates a new `Decorated<T>` from a `T`.
    pub fn new(value: T) -> Decorated<T> {
        Decorated {
            value,
            decor: Decor::default(),
            span: None,
        }
    }

    /// Consumes the `Decorated<T>` and returns the wrapped value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Returns a reference to the wrapped value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the wrapped value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> PartialEq for Decorated<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> AsRef<T> for Decorated<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.value()
    }
}

impl<T> AsMut<T> for Decorated<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.value_mut()
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

/// A wrapper type for a value together with its `ToString` representation with additional
/// [`Decor`] and span infromation attached.
#[derive(Debug, Clone, Eq)]
pub struct Formatted<T> {
    value: T,
    decor: Decor,
    repr: Option<RawString>,
    span: Option<Range<usize>>,
}

impl<T> Formatted<T>
where
    T: ToString,
{
    /// Creates a new `Formatted<T>` from a `T`.
    pub fn new(value: T) -> Formatted<T> {
        Formatted {
            value,
            decor: Decor::default(),
            repr: None,
            span: None,
        }
    }

    /// Returns the raw string representation of the value, if any.
    pub fn as_repr(&self) -> Option<&RawString> {
        self.repr.as_ref()
    }

    pub(crate) fn set_repr(&mut self, repr: impl Into<RawString>) {
        self.repr = Some(repr.into());
    }

    /// Formats the value using its `ToString` representation.
    pub fn format(&mut self) {
        self.set_repr(self.value.to_string())
    }
}

impl<T> Formatted<T> {
    /// Consumes the `Formatted<T>` and returns the wrapped value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Returns a reference to the wrapped value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the wrapped value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> PartialEq for Formatted<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> AsRef<T> for Formatted<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.value()
    }
}

impl<T> AsMut<T> for Formatted<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.value_mut()
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
    T: ToString,
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
