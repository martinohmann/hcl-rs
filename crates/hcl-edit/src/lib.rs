#![doc = include_str!("../README.md")]

extern crate alloc;

#[macro_use]
mod macros;

pub(crate) mod encode;
pub mod error;
pub mod expr;
pub mod parser;
pub mod repr;
pub mod structure;
pub mod template;
mod util;

pub use self::error::Error;
