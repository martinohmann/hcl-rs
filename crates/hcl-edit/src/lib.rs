#![doc = include_str!("../README.md")]

extern crate alloc;

pub mod error;
pub mod parser;
mod util;

pub use self::error::Error;
