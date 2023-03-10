use super::Rule;
use pest::error::LineColLocation;
use std::fmt;

/// The result type used by this module.
pub type ParseResult<T> = std::result::Result<T, Error>;

/// Error type returned when the parser encountered an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    fn new(inner: ErrorInner) -> Error {
        Error {
            inner: Box::new(inner),
        }
    }
    /// Returns the `Location` in the input where the error happened, if available.
    pub fn location(&self) -> Option<&Location> {
        self.inner.location.as_ref()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner.location {
            Some(loc) => {
                write!(
                    f,
                    "{} in line {}, col {}",
                    self.inner.msg, loc.line, loc.col
                )
            }
            None => write!(f, "{}", self.inner.msg),
        }
    }
}

impl From<crate::Error> for Error {
    fn from(err: crate::Error) -> Self {
        Error::new(ErrorInner {
            msg: err.to_string(),
            location: None,
        })
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(err: pest::error::Error<Rule>) -> Self {
        let (line, col) = match err.line_col {
            LineColLocation::Pos((l, c)) | LineColLocation::Span((l, c), (_, _)) => (l, c),
        };

        Error::new(ErrorInner {
            msg: err.to_string(),
            location: Some(Location { line, col }),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ErrorInner {
    msg: String,
    location: Option<Location>,
}

/// Represents a location in the parser input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// The one-based line number of the error.
    pub line: usize,
    /// The one-based column number of the error.
    pub col: usize,
}
