#![doc = include_str!("../README.md")]
#![allow(clippy::should_implement_trait)]
#![warn(missing_docs)]

pub mod de;
pub mod error;
#[macro_use]
mod macros;
pub mod format;
mod number;
mod parser;
pub mod ser;
pub mod structure;
#[cfg(test)]
mod tests;
pub mod value;

#[doc(inline)]
pub use de::{from_reader, from_slice, from_str};
#[doc(inline)]
pub use error::{Error, Result};
pub use number::Number;
pub use parser::parse;
#[doc(inline)]
pub use ser::{to_string, to_vec, to_writer};
#[doc(inline)]
pub use structure::{
    ser::to_expression, Attribute, Block, BlockBuilder, BlockLabel, Body, BodyBuilder, Expression,
    Object, ObjectKey, RawExpression, Structure,
};
#[doc(inline)]
pub use value::{Map, Value};
