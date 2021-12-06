//! The `Error` and `Result` types used by this crate.
use crate::parser::Rule;
use pest::{error::LineColLocation, Span};
use serde::{de, ser};
use std::fmt::{self, Display};
use std::io;

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
    /// Represents generic IO errors.
    IoError(io::Error),
}

impl Error {
    pub(crate) fn new<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::new_span(msg, None)
    }

    pub(crate) fn new_span<T>(msg: T, span: Option<Span<'_>>) -> Self
    where
        T: Display,
    {
        Self::Message {
            msg: msg.to_string(),
            location: span.map(Into::into),
        }
    }

    pub(crate) fn expected<T>(token: T) -> Self
    where
        T: Display,
    {
        Self::expected_span(token, None)
    }

    pub(crate) fn expected_span<T>(token: T, span: Option<Span<'_>>) -> Self
    where
        T: Display,
    {
        Self::new_span(format!("Expected `{}`", token), span)
    }

    pub(crate) fn with_span(self, span: Option<Span<'_>>) -> Self {
        match self {
            Error::Message { msg, location } => Error::Message {
                msg,
                location: match location {
                    Some(loc) => Some(loc),
                    None => span.map(Into::into),
                },
            },
            _ => self,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Eof => write!(f, "Unexpected end of input"),
            Error::IoError(err) => Display::fmt(err, f),
            Error::Message { msg, location } => match location {
                Some(loc) => {
                    write!(f, "{} in line {}, col {}", msg, loc.line, loc.col)
                }
                None => write!(f, "{}", msg),
            },
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(err: pest::error::Error<Rule>) -> Self {
        let (line, col) = match err.line_col {
            LineColLocation::Pos((l, c)) => (l, c),
            LineColLocation::Span((l, c), (_, _)) => (l, c),
        };

        Error::Message {
            msg: err.to_string(),
            location: Some(Location { line, col }),
        }
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
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    /// The one-based line number of the error.
    pub line: usize,
    /// The one-based column number of the error.
    pub col: usize,
}

impl From<Span<'_>> for Location {
    fn from(s: Span<'_>) -> Self {
        let (line, col) = s.start_pos().line_col();
        Self { line, col }
    }
}
