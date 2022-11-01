use crate::expr::Variable;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, Cow};
use std::fmt;
use std::ops;

/// Represents an HCL identifier.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename = "$hcl::identifier")]
pub struct Identifier(String);

impl Identifier {
    /// Create a new `Identifier` after validating that it only contains characters that are
    /// allowed in HCL identifiers.
    ///
    /// See [`Identifier::sanitized`][Identifier::sanitized] for an infallible alternative to this
    /// function.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl::Identifier;
    /// assert!(Identifier::new("some_ident").is_ok());
    /// assert!(Identifier::new("").is_err());
    /// assert!(Identifier::new("1two3").is_err());
    /// assert!(Identifier::new("with whitespace").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// If `ident` contains characters that are not allowed in HCL identifiers or if it is empty an
    /// error will be returned.
    pub fn new<T>(ident: T) -> Result<Self>
    where
        T: Into<String>,
    {
        let ident = ident.into();

        if ident.is_empty() {
            return Err(Error::InvalidIdentifier(ident));
        }

        let mut chars = ident.chars();
        let first = chars.next().unwrap();

        if !is_id_start(first) || !chars.all(is_id_continue) {
            return Err(Error::InvalidIdentifier(ident));
        }

        Ok(Identifier(ident))
    }

    /// Create a new `Identifier` after sanitizing the input if necessary.
    ///
    /// If `ident` contains characters that are not allowed in HCL identifiers will be sanitized
    /// according to the following rules:
    ///
    /// - An empty `ident` results in an identifier containing a single underscore.
    /// - Invalid characters in `ident` will be replaced with underscores.
    /// - If `ident` starts with a character that is invalid in the first position but would be
    ///   valid in the rest of an HCL identifier it is prefixed with an underscore.
    ///
    /// See [`Identifier::new`][Identifier::new] for a fallible alternative to this function if
    /// you prefer rejecting invalid identifiers instead of sanitizing them.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl::Identifier;
    /// assert_eq!(Identifier::sanitized("some_ident").as_str(), "some_ident");
    /// assert_eq!(Identifier::sanitized("").as_str(), "_");
    /// assert_eq!(Identifier::sanitized("1two3").as_str(), "_1two3");
    /// assert_eq!(Identifier::sanitized("with whitespace").as_str(), "with_whitespace");
    /// ```
    pub fn sanitized<T>(ident: T) -> Self
    where
        T: AsRef<str>,
    {
        let input = ident.as_ref();

        if input.is_empty() {
            return Identifier(String::from('_'));
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

        Identifier(ident)
    }

    /// Create a new `Identifier` without checking if it is valid.
    ///
    /// It is the caller's responsibility to ensure that the identifier is valid.
    ///
    /// For most use cases [`Identifier::new`][Identifier::new] or
    /// [`Identifier::sanitized`][Identifier::sanitized] should be preferred.
    ///
    /// # Safety
    ///
    /// This function is not marked as unsafe because it does not cause undefined behaviour.
    /// However, attempting to serialize an invalid identifier to HCL will produce invalid output.
    pub fn unchecked<T>(ident: T) -> Self
    where
        T: Into<String>,
    {
        Identifier(ident.into())
    }

    /// Consume `self` and return the wrapped `String`.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Return a reference to the wrapped `str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[inline]
fn is_id_start(ch: char) -> bool {
    ch == '_' || unicode_ident::is_xid_start(ch)
}

#[inline]
fn is_id_continue(ch: char) -> bool {
    ch == '-' || unicode_ident::is_xid_continue(ch)
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Identifier::sanitized(s)
    }
}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Identifier::sanitized(s)
    }
}

impl<'a> From<Cow<'a, str>> for Identifier {
    fn from(s: Cow<'a, str>) -> Self {
        Identifier::sanitized(s)
    }
}

impl From<Variable> for Identifier {
    fn from(variable: Variable) -> Self {
        variable.into_inner()
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl ops::Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
