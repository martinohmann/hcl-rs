#![doc = include_str!("../README.md")]
#![allow(clippy::should_implement_trait)]
#![warn(missing_docs)]

pub mod de;
pub mod error;
mod number;
mod parser;
pub mod structure;
pub mod value;

pub use de::{from_reader, from_str};
pub use error::{Error, Result};
pub use number::Number;
pub use parser::parse;
pub use structure::{Attribute, Block, BlockBuilder, BlockLabel, Body, BodyBuilder, Structure};
pub use value::{Map, Value};
