use crate::encode::EncodeState;
use hcl_primitives::InternalString;
use std::fmt::Write;
use std::ops::Range;

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
