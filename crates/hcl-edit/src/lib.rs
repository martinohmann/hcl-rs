#![doc = include_str!("../README.md")]

extern crate alloc;

#[macro_use]
mod macros;

pub(crate) mod encode;
pub mod error;
pub mod expr;
pub mod parser;
mod raw_string;
pub mod repr;
pub mod structure;
pub mod template;
mod util;

#[doc(inline)]
pub use self::raw_string::RawString;

use self::error::Error;

/// Core concepts available for glob import.
pub mod prelude {
    pub use crate::repr::{Decorate, Span};
}

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::{Ident, InternalString, Number};
