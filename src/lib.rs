#![doc = include_str!("../README.md")]
#![allow(clippy::should_implement_trait)]
#![warn(missing_docs)]

pub mod de;
pub mod error;
mod number;
mod parser;
pub mod structure;
pub mod value;

#[doc(inline)]
pub use de::{from_reader, from_slice, from_str};
#[doc(inline)]
pub use error::{Error, Result};
pub use number::Number;
pub use parser::parse;
#[doc(inline)]
pub use structure::{
    Attribute, Block, BlockBuilder, BlockLabel, Body, BodyBuilder, Expression, Object, ObjectKey,
    RawExpression, Structure,
};
#[doc(inline)]
pub use value::{Map, Value};
