#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

extern crate alloc;

pub mod error;
pub mod parser;
mod util;

pub use self::error::Error;
