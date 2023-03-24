#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

extern crate alloc;

#[macro_use]
mod macros;

pub(crate) mod encode;
pub mod expr;
pub mod parser;
mod raw_string;
pub mod repr;
pub mod structure;
pub mod template;
mod util;

#[doc(inline)]
pub use self::raw_string::RawString;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::{Ident, Number};

mod private {
    pub trait Sealed {}
}
