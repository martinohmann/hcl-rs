#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod error;
pub mod ident;
mod internal_string;
mod number;

#[doc(inline)]
pub use self::error::Error;

#[doc(inline)]
pub use self::ident::Ident;

#[doc(inline)]
pub use self::internal_string::InternalString;

#[doc(inline)]
pub use self::number::Number;
