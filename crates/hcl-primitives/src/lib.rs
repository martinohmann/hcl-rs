#![doc = include_str!("../README.md")]
#![warn(missing_docs, clippy::pedantic)]
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc,
    clippy::needless_lifetimes
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "serde")]
mod de;
mod error;
pub mod expr;
pub mod ident;
mod internal_string;
mod number;
pub mod template;

#[doc(inline)]
pub use self::error::Error;

#[doc(inline)]
pub use self::ident::Ident;

#[doc(inline)]
pub use self::internal_string::InternalString;

#[doc(inline)]
pub use self::number::Number;
