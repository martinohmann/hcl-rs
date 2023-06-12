#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::bool_to_int_with_if,
    clippy::let_underscore_untyped,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::naive_bytecount,
    clippy::return_self_not_must_use
)]

extern crate alloc;

#[macro_use]
mod macros;

mod encode;
pub mod expr;
pub mod parser;
mod raw_string;
pub mod repr;
pub mod structure;
pub mod template;
mod util;
pub mod visit;
pub mod visit_mut;

#[doc(inline)]
pub use self::raw_string::RawString;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::{Ident, Number};

/// Core concepts available for glob import.
///
/// This includes useful traits like [`Decorate`](crate::repr::Decorate) and
/// [`Span`](crate::repr::Span).
///
/// # Example
///
/// ```
/// use hcl_edit::expr::Expression;
/// use hcl_edit::prelude::*;
///
/// let mut expr = Expression::from("A string");
/// expr.decor_mut().set_suffix(" // Comment.");
/// assert_eq!(expr.to_string(), r#""A string" // Comment."#);
/// ```
pub mod prelude {
    pub use crate::repr::{Decorate, Span};
}
