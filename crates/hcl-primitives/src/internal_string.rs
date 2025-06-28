//! Provides the `InternalString` type and associated functionality.

use alloc::borrow::{Borrow, Cow};
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::fmt;
use core::ops::Deref;

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

    /// Converts the `InternalString` into a mutable string type.
    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        #[cfg(feature = "perf")]
        let string = self.0.into_string();
        #[cfg(not(feature = "perf"))]
        let string = self.0;

        string
    }

    /// Converts the `InternalString` into a copy-on-write string type.
    #[inline]
    #[must_use]
    pub fn into_cow_str(self) -> Cow<'static, str> {
        #[cfg(feature = "perf")]
        let cow = self.0.into_cow_str();
        #[cfg(not(feature = "perf"))]
        let cow = self.0.into();

        cow
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
        is.into_string()
    }
}

impl From<InternalString> for Cow<'_, str> {
    fn from(value: InternalString) -> Self {
        value.into_cow_str()
    }
}

impl<'a> From<&'a InternalString> for Cow<'a, str> {
    fn from(value: &'a InternalString) -> Self {
        Cow::Borrowed(value.as_str())
    }
}

impl fmt::Debug for InternalString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InternalString({self})")
    }
}

impl fmt::Display for InternalString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for InternalString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for InternalString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Inner::deserialize(deserializer).map(InternalString)
    }
}
