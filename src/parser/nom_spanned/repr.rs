#![allow(missing_docs)]
#![allow(dead_code)]

use kstring::KString;
use std::borrow::Borrow;
use std::fmt;
use std::ops::{Deref, Range};

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternalString(KString);

impl InternalString {
    /// Create an empty string
    pub fn new() -> Self {
        InternalString(KString::new())
    }

    /// Access the underlying string
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for InternalString {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Deref for InternalString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for InternalString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for InternalString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for InternalString {
    #[inline]
    fn from(s: &str) -> Self {
        InternalString(KString::from_ref(s))
    }
}

impl From<String> for InternalString {
    #[inline]
    fn from(s: String) -> Self {
        InternalString(s.into())
    }
}

impl From<&String> for InternalString {
    #[inline]
    fn from(s: &String) -> Self {
        InternalString(s.into())
    }
}

impl From<&InternalString> for InternalString {
    #[inline]
    fn from(s: &InternalString) -> Self {
        s.clone()
    }
}

impl From<Box<str>> for InternalString {
    #[inline]
    fn from(s: Box<str>) -> Self {
        InternalString(s.into())
    }
}

impl fmt::Display for InternalString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawString(RawStringInner);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawStringInner {
    Empty,
    Spanned(Range<usize>),
    Explicit(InternalString),
}

impl RawString {
    pub(crate) fn from_span(span: Range<usize>) -> Self {
        if span.is_empty() {
            RawString(RawStringInner::Empty)
        } else {
            RawString(RawStringInner::Spanned(span))
        }
    }

    pub fn span(&self) -> Option<Range<usize>> {
        match &self.0 {
            RawStringInner::Empty => None,
            RawStringInner::Explicit(_) => None,
            RawStringInner::Spanned(span) => Some(span.clone()),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.0 {
            RawStringInner::Empty => Some(""),
            RawStringInner::Explicit(s) => Some(s.0.as_str()),
            RawStringInner::Spanned(_) => None,
        }
    }
}

impl Default for RawString {
    fn default() -> Self {
        RawString(RawStringInner::Empty)
    }
}

impl From<Range<usize>> for RawString {
    fn from(span: Range<usize>) -> Self {
        RawString::from_span(span)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Decor {
    pub prefix: Option<RawString>,
    pub suffix: Option<RawString>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formatted<T> {
    value: T,
    span: Range<usize>,
    decor: Decor,
}

impl<T> Formatted<T> {
    pub fn new(value: T, span: Range<usize>) -> Formatted<T> {
        Formatted::new_with_decor(value, span, Decor::default())
    }

    pub fn new_with_decor(value: T, span: Range<usize>, decor: Decor) -> Formatted<T> {
        Formatted { value, span, decor }
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn decor(&self) -> &Decor {
        &self.decor
    }

    pub fn decor_mut(&mut self) -> &mut Decor {
        &mut self.decor
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn value_into<U>(self) -> U
    where
        T: Into<U>,
    {
        self.value.into()
    }
}

impl<T> From<(T, Range<usize>)> for Formatted<T> {
    fn from((value, span): (T, Range<usize>)) -> Self {
        Formatted::new(value, span)
    }
}

impl<T> From<(T, Range<usize>, Decor)> for Formatted<T> {
    fn from((value, span, decor): (T, Range<usize>, Decor)) -> Self {
        Formatted::new_with_decor(value, span, decor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    value: T,
    span: Range<usize>,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Range<usize>) -> Spanned<T> {
        Spanned { value, span }
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn value_into<U>(self) -> U
    where
        T: Into<U>,
    {
        self.value.into()
    }
}

impl<T> From<(T, Range<usize>)> for Spanned<T> {
    fn from((value, span): (T, Range<usize>)) -> Self {
        Spanned::new(value, span)
    }
}
