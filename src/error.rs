//! The `Error` and `Result` types used by this crate.
use crate::eval;
#[cfg(feature = "pest")]
use crate::parser::pest::Rule;
#[cfg(feature = "pest")]
use pest::{error::LineColLocation, Span};
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
    /// Represents a generic error message with optional location.
    Message {
        /// The error message.
        msg: String,
        /// An optional location context where the error happened in the input.
        location: Option<Location>,
    },
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
}

impl Error {
    pub(crate) fn new<T>(msg: T) -> Error
    where
        T: Display,
    {
        Error::Message {
            msg: msg.to_string(),
            location: None,
        }
    }

    /// Returns the `Location` in the input where the error happened, if available.
    pub fn location(&self) -> Option<&Location> {
        match self {
            Error::Message { location, .. } => location.as_ref(),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Eof => write!(f, "unexpected end of input"),
            Error::Io(err) => Display::fmt(err, f),
            Error::Utf8(err) => Display::fmt(err, f),
            Error::Message { msg, location } => match location {
                Some(loc) => {
                    write!(f, "{msg} in line {}, col {}", loc.line, loc.col)
                }
                None => write!(f, "{msg}"),
            },
            Error::InvalidEscape(c) => write!(f, "invalid escape sequence '\\{c}'"),
            Error::InvalidUnicodeCodePoint(u) => {
                write!(f, "invalid unicode code point '\\u{u}'")
            }
            Error::InvalidIdentifier(ident) => write!(f, "invalid identifier `{ident}`"),
            Error::Eval(err) => write!(f, "eval error: {err}"),
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

#[cfg(feature = "pest")]
impl From<pest::error::Error<Rule>> for Error {
    fn from(err: pest::error::Error<Rule>) -> Self {
        let (line, col) = match err.line_col {
            LineColLocation::Pos((l, c)) | LineColLocation::Span((l, c), (_, _)) => (l, c),
        };

        Error::Message {
            msg: err.to_string(),
            location: Some(Location { line, col }),
        }
    }
}

impl From<eval::Error> for Error {
    fn from(err: eval::Error) -> Self {
        Error::Eval(err)
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

/// One-based line and column at which the error was detected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    /// The one-based line number of the error.
    pub line: usize,
    /// The one-based column number of the error.
    pub col: usize,
}

#[cfg(feature = "pest")]
impl From<Span<'_>> for Location {
    fn from(span: Span<'_>) -> Self {
        let (line, col) = span.start_pos().line_col();
        Location { line, col }
    }
}
