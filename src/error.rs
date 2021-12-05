//! The `Error` and `Result` types used by this crate.
use serde::{de, ser};
use std::fmt::Display;
use thiserror::Error;

/// The result type used by this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error type used by this crate.
#[derive(Error, Debug)]
pub enum Error {
    /// Represents the error emitted when parsing HCL data fails.
    #[error("HCL parse error:\n{0}")]
    ParseError(String),
    /// Represents a generic error message.
    #[error("{0}")]
    Message(String),
    /// Represents the error emitted when the `Deserializer` hits an unexpected end of input.
    #[error("Unexpected end of input")]
    Eof,
    /// Represents the error emitted on syntax errors.
    #[error("Syntax error")]
    Syntax,
    /// Represents the error emitted when an unexpected token is encountered during
    /// deserialization.
    #[error("Token expected `{0}`")]
    TokenExpected(String),
    /// Represents generic IO errors.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl Error {
    pub(crate) fn token_expected<S>(s: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::TokenExpected(s.as_ref().into())
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
