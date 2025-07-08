#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs, clippy::pedantic)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::enum_glob_use,
    clippy::let_underscore_untyped,
    clippy::match_wildcard_for_single_variants,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_lifetimes,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::return_self_not_must_use,
    clippy::should_implement_trait,
    clippy::struct_excessive_bools,
    clippy::unnecessary_wraps,
    clippy::wildcard_imports
)]

#[macro_use]
mod macros;

pub mod de;
pub mod error;
pub mod eval;
pub mod expr;
pub mod format;
mod ident;
pub mod ser;
pub mod structure;
pub mod template;
#[cfg(test)]
mod tests;
mod util;
pub mod value;

pub use hcl_edit as edit;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::{InternalString, Number};

#[doc(inline)]
pub use de::{from_body, from_reader, from_slice, from_str};

#[doc(inline)]
pub use error::{Error, Result};

#[doc(inline)]
pub use expr::{to_expression, Expression, Object, ObjectKey};

// Deprecated, these re-exports will be removed in a future release.
#[doc(hidden)]
pub use expr::{
    BinaryOp, BinaryOperator, Conditional, ForExpr, FuncCall, FuncCallBuilder, Heredoc,
    HeredocStripMode, Operation, TemplateExpr, Traversal, TraversalOperator, UnaryOp,
    UnaryOperator, Variable,
};

pub use ident::Identifier;

#[doc(inline)]
pub use ser::{to_string, to_vec, to_writer};

#[doc(inline)]
pub use structure::{Attribute, Block, BlockLabel, Body, Structure};

// Deprecated, these re-exports will be removed in a future release.
#[doc(hidden)]
pub use structure::{BlockBuilder, BodyBuilder};

#[doc(inline)]
pub use template::Template;

#[doc(inline)]
pub use value::{from_value, to_value, Map, Value};

/// Parse a `hcl::Body` from a `&str`.
///
/// If deserialization into a different type is preferred consider using [`hcl::from_str`][from_str].
///
/// [from_str]: ./de/fn.from_str.html
///
/// # Example
///
/// ```
/// use hcl::{Attribute, Block, Body};
///
/// let input = r#"
///     some_attr = "foo"
///
///     some_block "some_block_label" {
///       attr = "value"
///     }
/// "#;
///
/// let expected = Body::builder()
///     .add_attribute(("some_attr", "foo"))
///     .add_block(
///         Block::builder("some_block")
///             .add_label("some_block_label")
///             .add_attribute(("attr", "value"))
///             .build()
///     )
///     .build();
///
/// let body = hcl::parse(input)?;
///
/// assert_eq!(body, expected);
/// # Ok::<(), Box<dyn core::error::Error>>(())
/// ```
///
/// # Errors
///
/// This function fails with an error if the `input` cannot be parsed as HCL.
#[inline]
pub fn parse(input: &str) -> Result<Body> {
    input.parse()
}
