//! Construct and validate HCL identifiers.

use crate::{Error, InternalString};
use alloc::borrow::{Borrow, Cow};
use alloc::format;
use alloc::string::String;
use core::fmt;
use core::ops;
use core::str::FromStr;

/// Represents an HCL identifier.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ident(InternalString);

impl Ident {
    /// Create a new `Ident` after validating that it only contains characters that are allowed in
    /// HCL identifiers.
    ///
    /// See [`Ident::new_sanitized`][Ident::new_sanitized] for an infallible alternative to this
    /// function.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl_primitives::Ident;
    /// assert!(Ident::new("some_ident").is_ok());
    /// assert!(Ident::new("").is_err());
    /// assert!(Ident::new("1two3").is_err());
    /// assert!(Ident::new("with whitespace").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// If `ident` contains characters that are not allowed in HCL identifiers or if it is empty an
    /// error will be returned.
    pub fn new<T>(ident: T) -> Result<Ident, Error>
    where
        T: Into<InternalString>,
    {
        let ident = ident.into();

        if !is_ident(&ident) {
            return Err(Error::new(format!("invalid identifier `{ident}`")));
        }

        Ok(Ident(ident))
    }

    /// Create a new `Ident` after sanitizing the input if necessary.
    ///
    /// If `ident` contains characters that are not allowed in HCL identifiers will be sanitized
    /// according to the following rules:
    ///
    /// - An empty `ident` results in an identifier containing a single underscore.
    /// - Invalid characters in `ident` will be replaced with underscores.
    /// - If `ident` starts with a character that is invalid in the first position but would be
    ///   valid in the rest of an HCL identifier it is prefixed with an underscore.
    ///
    /// See [`Ident::new`][Ident::new] for a fallible alternative to this function if you prefer
    /// rejecting invalid identifiers instead of sanitizing them.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl_primitives::Ident;
    /// assert_eq!(Ident::new_sanitized("some_ident").as_str(), "some_ident");
    /// assert_eq!(Ident::new_sanitized("").as_str(), "_");
    /// assert_eq!(Ident::new_sanitized("1two3").as_str(), "_1two3");
    /// assert_eq!(Ident::new_sanitized("with whitespace").as_str(), "with_whitespace");
    /// ```
    pub fn new_sanitized<T>(ident: T) -> Self
    where
        T: AsRef<str>,
    {
        let input = ident.as_ref();

        if input.is_empty() {
            return Ident(InternalString::from("_"));
        }

        let mut ident = String::with_capacity(input.len());

        for (i, ch) in input.chars().enumerate() {
            if i == 0 && is_id_start(ch) {
                ident.push(ch);
            } else if is_id_continue(ch) {
                if i == 0 {
                    ident.push('_');
                }
                ident.push(ch);
            } else {
                ident.push('_');
            }
        }

        Ident(InternalString::from(ident))
    }

    /// Create a new `Ident` without checking if it is valid.
    ///
    /// It is the caller's responsibility to ensure that the identifier is valid.
    ///
    /// For most use cases [`Ident::new`][Ident::new] or
    /// [`Ident::new_sanitized`][Ident::new_sanitized] should be preferred.
    ///
    /// This function is not marked as unsafe because it does not cause undefined behaviour.
    /// However, attempting to serialize an invalid identifier to HCL will produce invalid output.
    #[inline]
    pub fn new_unchecked<T>(ident: T) -> Self
    where
        T: Into<InternalString>,
    {
        Ident(ident.into())
    }

    /// Converts the `Ident` to a mutable string type.
    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        self.0.into_string()
    }

    /// Return a reference to the wrapped `str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<InternalString> for Ident {
    type Error = Error;

    #[inline]
    fn try_from(s: InternalString) -> Result<Self, Self::Error> {
        Ident::new(s)
    }
}

impl TryFrom<String> for Ident {
    type Error = Error;

    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ident::new(s)
    }
}

impl TryFrom<&str> for Ident {
    type Error = Error;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ident::new(s)
    }
}

impl<'a> TryFrom<Cow<'a, str>> for Ident {
    type Error = Error;

    #[inline]
    fn try_from(s: Cow<'a, str>) -> Result<Self, Self::Error> {
        Ident::new(s)
    }
}

impl FromStr for Ident {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ident::new(s)
    }
}

impl From<Ident> for InternalString {
    #[inline]
    fn from(ident: Ident) -> Self {
        ident.0
    }
}

impl From<Ident> for String {
    #[inline]
    fn from(ident: Ident) -> Self {
        ident.into_string()
    }
}

impl fmt::Debug for Ident {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ident({self})")
    }
}

impl fmt::Display for Ident {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ops::Deref for Ident {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Ident {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Ident {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Ident {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Ident {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = InternalString::deserialize(deserializer)?;
        Ident::new(string).map_err(serde::de::Error::custom)
    }
}

/// Determines if `ch` is a valid HCL identifier start character.
///
/// # Example
///
/// ```
/// # use hcl_primitives::ident::is_id_start;
/// assert!(is_id_start('_'));
/// assert!(is_id_start('a'));
/// assert!(!is_id_start('-'));
/// assert!(!is_id_start('1'));
/// assert!(!is_id_start(' '));
/// ```
#[inline]
pub fn is_id_start(ch: char) -> bool {
    unicode_ident::is_xid_start(ch) || ch == '_'
}

/// Determines if `ch` is a valid HCL identifier continue character.
///
/// # Example
///
/// ```
/// # use hcl_primitives::ident::is_id_continue;
/// assert!(is_id_continue('-'));
/// assert!(is_id_continue('_'));
/// assert!(is_id_continue('a'));
/// assert!(is_id_continue('1'));
/// assert!(!is_id_continue(' '));
/// ```
#[inline]
pub fn is_id_continue(ch: char) -> bool {
    unicode_ident::is_xid_continue(ch) || ch == '-'
}

/// Determines if `s` represents a valid HCL identifier.
///
/// A string is a valid HCL identifier if:
///
/// - [`is_id_start`] returns `true` for the first character, and
/// - [`is_id_continue`] returns `true` for all remaining chacters
///
/// # Example
///
/// ```
/// # use hcl_primitives::ident::is_ident;
/// assert!(!is_ident(""));
/// assert!(!is_ident("-foo"));
/// assert!(!is_ident("123foo"));
/// assert!(!is_ident("foo bar"));
/// assert!(is_ident("fööbär"));
/// assert!(is_ident("foobar123"));
/// assert!(is_ident("FOO-bar123"));
/// assert!(is_ident("foo_BAR123"));
/// assert!(is_ident("_foo"));
/// assert!(is_ident("_123"));
/// assert!(is_ident("foo_"));
/// assert!(is_ident("foo-"));
/// ```
#[inline]
pub fn is_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    is_id_start(first) && chars.all(is_id_continue)
}
