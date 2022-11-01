#![doc = include_str!("../README.md")]
#![allow(clippy::should_implement_trait)]
#![warn(missing_docs)]

#[macro_use]
mod macros;

pub mod de;
pub mod error;
pub mod eval;
pub mod expr;
pub mod format;
mod ident;
mod number;
mod parser;
pub mod ser;
pub mod structure;
pub mod template;
#[cfg(test)]
mod tests;
mod util;
pub mod value;

#[doc(inline)]
pub use de::{from_body, from_reader, from_slice, from_str};
#[doc(inline)]
pub use error::{Error, Result};
#[doc(inline)]
pub use expr::{
    BinaryOp, BinaryOperator, Conditional, Expression, ForExpr, FuncCall, FuncCallBuilder, Heredoc,
    HeredocStripMode, Object, ObjectKey, Operation, RawExpression, TemplateExpr, Traversal,
    TraversalOperator, UnaryOp, UnaryOperator, Variable,
};
pub use ident::Identifier;
pub use number::Number;
pub use parser::parse;
#[doc(inline)]
pub use ser::{to_expression, to_string, to_vec, to_writer};
#[doc(inline)]
pub use structure::{Attribute, Block, BlockBuilder, BlockLabel, Body, BodyBuilder, Structure};
#[doc(inline)]
pub use value::{Map, Value};
