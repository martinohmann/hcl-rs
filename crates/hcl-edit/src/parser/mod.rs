//! HCL parser implementation.

mod winnow;

pub use self::winnow::{parse, parse_template, Error};

/// The result type used by this module.
pub type ParseResult<T> = std::result::Result<T, Error>;

/// Represents a location in the parser input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// The one-based line number of the error.
    pub line: usize,
    /// The one-based column number of the error.
    pub col: usize,
}
