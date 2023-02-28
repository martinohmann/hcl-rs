//! HCL parser implementation.

#[cfg(feature = "winnow")]
#[path = "winnow/mod.rs"]
mod imp;

#[cfg(feature = "pest")]
#[path = "pest/mod.rs"]
mod imp;

#[cfg(feature = "winnow")]
pub use self::imp::parse_raw;
pub(crate) use self::imp::parse_template;
pub use self::imp::{parse, Error};

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
