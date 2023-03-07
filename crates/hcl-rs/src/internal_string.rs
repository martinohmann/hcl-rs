use std::borrow::{Borrow, Cow};
use std::fmt;
use std::ops::Deref;

#[cfg(feature = "perf")]
type Inner = kstring::KString;
#[cfg(not(feature = "perf"))]
type Inner = String;

/// An opaque string storage which inlines small strings on the stack if the `perf` feature is
/// enabled.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternalString(Inner);

impl InternalString {
    /// Create a new empty `InternalString`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        #[cfg(feature = "perf")]
        let inner = kstring::KString::EMPTY;
        #[cfg(not(feature = "perf"))]
        let inner = String::new();

        InternalString(inner)
    }

    /// Returns a reference to the underlying string.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for InternalString {
    fn default() -> Self {
        InternalString::new()
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
        #[cfg(feature = "perf")]
        let inner = kstring::KString::from_ref(s);
        #[cfg(not(feature = "perf"))]
        let inner = String::from(s);

        InternalString(inner)
    }
}

impl From<String> for InternalString {
    #[inline]
    fn from(s: String) -> Self {
        #[cfg(feature = "perf")]
        let inner = kstring::KString::from_string(s);
        #[cfg(not(feature = "perf"))]
        let inner = s;

        InternalString(inner)
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

impl<'a> From<Cow<'a, str>> for InternalString {
    #[inline]
    fn from(s: Cow<'a, str>) -> Self {
        match s {
            Cow::Borrowed(borrowed) => borrowed.into(),
            Cow::Owned(owned) => owned.into(),
        }
    }
}

impl From<InternalString> for String {
    #[inline]
    fn from(is: InternalString) -> Self {
        #[cfg(feature = "perf")]
        let string = is.0.to_string();
        #[cfg(not(feature = "perf"))]
        let string = is.0;

        string
    }
}

impl fmt::Debug for InternalString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for InternalString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl serde::Serialize for InternalString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for InternalString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Inner::deserialize(deserializer).map(InternalString)
    }
}
