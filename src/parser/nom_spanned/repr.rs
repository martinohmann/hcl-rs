#![allow(missing_docs)]

use kstring::KString;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalString(pub(crate) KString);

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

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

impl<T> From<(T, Range<usize>)> for Formatted<T> {
    fn from((value, span): (T, Range<usize>)) -> Self {
        Formatted::new(value, span)
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
}

impl<T> From<(T, Range<usize>)> for Spanned<T> {
    fn from((value, span): (T, Range<usize>)) -> Self {
        Spanned::new(value, span)
    }
}
