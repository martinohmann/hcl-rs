use crate::encode::EncodeState;
use hcl_primitives::InternalString;
use std::borrow::Cow;
use std::fmt::Write;
use std::ops::{self, Range};

/// Opaque string storage for raw HCL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawString(RawStringInner);

#[derive(Debug, Clone, PartialEq, Eq)]
enum RawStringInner {
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

    pub(crate) fn span(&self) -> Option<Range<usize>> {
        match &self.0 {
            RawStringInner::Empty | RawStringInner::Explicit(_) => None,
            RawStringInner::Spanned(span) => Some(span.clone()),
        }
    }

    /// Returns the `RawString` as a `&str`.
    pub(crate) fn as_str(&self) -> &str {
        match &self.0 {
            RawStringInner::Empty | RawStringInner::Spanned(_) => "",
            RawStringInner::Explicit(s) => s.as_str(),
        }
    }

    pub(crate) fn is_multiline(&self) -> bool {
        self.as_str().contains('\n')
    }

    pub(crate) fn encode_with_default(
        &self,
        buf: &mut EncodeState,
        default: &str,
    ) -> std::fmt::Result {
        if let RawStringInner::Spanned(_) = self.0 {
            buf.write_str(default)
        } else {
            buf.write_str(self.as_str())
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        match &self.0 {
            RawStringInner::Empty | RawStringInner::Explicit(_) => {}
            RawStringInner::Spanned(span) => {
                *self = RawString::from(input.get(span.clone()).unwrap_or_else(|| {
                    panic!("span {span:?} should be in input:\n```\n{input}\n```")
                }));
            }
        }
    }
}

impl Default for RawString {
    fn default() -> Self {
        RawString(RawStringInner::Empty)
    }
}

impl ops::Deref for RawString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&str> for RawString {
    #[inline]
    fn from(s: &str) -> Self {
        if s.is_empty() {
            RawString(RawStringInner::Empty)
        } else {
            RawString::from(InternalString::from(s))
        }
    }
}

impl<'a> From<Cow<'a, str>> for RawString {
    #[inline]
    fn from(s: Cow<'a, str>) -> Self {
        if s.is_empty() {
            RawString(RawStringInner::Empty)
        } else {
            RawString::from(InternalString::from(s))
        }
    }
}

impl From<String> for RawString {
    #[inline]
    fn from(s: String) -> Self {
        if s.is_empty() {
            RawString(RawStringInner::Empty)
        } else {
            RawString::from(InternalString::from(s))
        }
    }
}

impl From<InternalString> for RawString {
    #[inline]
    fn from(inner: InternalString) -> Self {
        RawString(RawStringInner::Explicit(inner))
    }
}

impl<'a> From<RawString> for Cow<'a, str> {
    #[inline]
    fn from(s: RawString) -> Self {
        match s.0 {
            RawStringInner::Empty | RawStringInner::Spanned(_) => Cow::Borrowed(""),
            RawStringInner::Explicit(s) => Cow::Owned(s.into_string()),
        }
    }
}

impl<'a> From<&'a RawString> for Cow<'a, str> {
    #[inline]
    fn from(s: &'a RawString) -> Self {
        match &s.0 {
            RawStringInner::Empty | RawStringInner::Spanned(_) => Cow::Borrowed(""),
            RawStringInner::Explicit(s) => Cow::Borrowed(s.as_str()),
        }
    }
}
