use crate::{Identifier, Result};
use serde::Deserialize;
use std::ops::Deref;

/// A type representing a variable in an HCL expression.
///
/// It is a wrapper around the [`Identifier`][crate::Identifier] type and behaves the same in most
/// cases via its `Deref` implementation.
///
/// This is a separate type to differentiate between bare identifiers and variable identifiers
/// which have different semantics in different scopes.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Variable(Identifier);

impl Variable {
    /// Create a new `Variable` after validating that it only contains characters that are allowed
    /// in HCL identifiers.
    ///
    /// See the documentation of [`Identifier::new`][crate::Identifier::new] for more.
    ///
    /// # Errors
    ///
    /// If `ident` contains characters that are not allowed in HCL identifiers or if it is empty an
    /// error will be returned.
    pub fn new<T>(ident: T) -> Result<Self>
    where
        T: Into<String>,
    {
        Identifier::new(ident).map(Variable)
    }

    /// Create a new `Variable` after sanitizing the input if necessary.
    ///
    /// See the documentation of [`Identifier::sanitized`][crate::Identifier::sanitized] for more.
    pub fn sanitized<T>(ident: T) -> Self
    where
        T: AsRef<str>,
    {
        Variable(Identifier::sanitized(ident))
    }

    /// Create a new `Variable` from an identifier without checking if it is valid in HCL.
    ///
    /// It is the caller's responsibility to ensure that the variable identifier is valid.
    ///
    /// See the documentation of [`Identifier::unchecked`][crate::Identifier::unchecked] for more.
    ///
    /// # Safety
    ///
    /// This function is not marked as unsafe because it does not cause undefined behaviour.
    /// However, attempting to serialize an invalid variable identifier to HCL will produce invalid
    /// output.
    pub fn unchecked<T>(ident: T) -> Self
    where
        T: Into<String>,
    {
        Variable(Identifier::unchecked(ident))
    }

    /// Consume `self` and return the wrapped `Identifier`.
    pub fn into_inner(self) -> Identifier {
        self.0
    }
}

impl Deref for Variable {
    type Target = Identifier;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Identifier> for Variable {
    fn from(ident: Identifier) -> Self {
        Variable(ident)
    }
}
