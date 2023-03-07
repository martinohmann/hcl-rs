//! The `Error` and `Result` types used by this crate.
use crate::eval;
use crate::parser;
use serde::{de, ser};
use std::fmt::{self, Display};
use std::io;
use std::str::Utf8Error;

/// The result type used by this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error type used by this crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Represents a generic error message.
    Message(String),
    /// Represents the error emitted when the `Deserializer` hits an unexpected end of input.
    Eof,
    /// Represents an error that resulted from invalid UTF8 input.
    Utf8(Utf8Error),
    /// Represents generic IO errors.
    Io(io::Error),
    /// Represents errors due to invalid escape characters that may occur when unescaping
    /// user-provided strings.
    InvalidEscape(char),
    /// Represents errors due to invalid unicode code points that may occur when unescaping
    /// user-provided strings.
    InvalidUnicodeCodePoint(String),
    /// Represents errors that resulted from identifiers that are not valid in HCL.
    InvalidIdentifier(String),
    /// Represents errors during expression evaluation.
    Eval(eval::Error),
    /// Represents parsing errors.
    Parse(parser::Error),
}

impl Error {
    pub(crate) fn new<T>(msg: T) -> Error
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::Eof => write!(f, "unexpected end of input"),
            Error::Io(err) => Display::fmt(err, f),
            Error::Utf8(err) => Display::fmt(err, f),
            Error::InvalidEscape(c) => write!(f, "invalid escape sequence '\\{c}'"),
            Error::InvalidUnicodeCodePoint(u) => {
                write!(f, "invalid unicode code point '\\u{u}'")
            }
            Error::InvalidIdentifier(ident) => write!(f, "invalid identifier `{ident}`"),
            Error::Eval(err) => write!(f, "eval error: {err}"),
            Error::Parse(err) => write!(f, "{err}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Error::Utf8(err)
    }
}

impl From<eval::Error> for Error {
    fn from(err: eval::Error) -> Self {
        Error::Eval(err)
    }
}

impl From<parser::Error> for Error {
    fn from(err: parser::Error) -> Self {
        Error::Parse(err)
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new(msg)
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new(msg)
    }
}
