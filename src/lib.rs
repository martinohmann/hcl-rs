//! This crate provides functionality to deserialize and manipulate HCL data.
//!
//! Serializing to HCL is not supported.

#![allow(clippy::should_implement_trait)]
#![warn(missing_docs)]

pub mod de;
pub mod error;
mod number;
mod parser;
pub mod value;

pub use de::{from_reader, from_str};
pub use error::{Error, Result};
pub use number::Number;
pub use value::{Map, Value};
