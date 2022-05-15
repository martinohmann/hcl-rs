#![doc = include_str!("../README.md")]
#![allow(clippy::should_implement_trait)]
#![warn(missing_docs)]

pub mod de;
pub mod error;
mod number;
mod parser;
pub mod structure;
pub mod value;

pub use de::{from_reader, from_slice, from_str};
pub use error::{Error, Result};
pub use number::Number;
pub use parser::parse;
pub use structure::{
    Attribute, Block, BlockBuilder, BlockLabel, Body, BodyBuilder, Expression, Object, ObjectKey,
    RawExpression, Structure,
};
pub use value::{Map, Value};

trait OptionExt<T> {
    /// Takes the value out of an `Option` and leaves `None` in place. This is a shorthand for the
    /// pattern `.take().unwrap()`.
    ///
    /// Panics if the `Option` is `None`.
    fn consume(&mut self) -> T;
}

impl<T> OptionExt<T> for Option<T> {
    fn consume(&mut self) -> T {
        self.take().unwrap()
    }
}
