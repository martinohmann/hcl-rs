//! Types to represent the HCL structural sub-language.
//!
//! The main types in this module are:
//!
//! - [`Attribute`]: represent an HCL attribute
//! - [`Block`]: represent an HCL block
//! - [`BlockBuilder`]: provides functionality for building `Block`s
//! - [`Body`]: represent the body of an HCL configuration or block
//! - [`BodyBuilder`]: provides functionality for building `Body`s
//!
//! ## Examples
//!
//! Building HCL structures:
//!
//! ```
//! use hcl::{Body, Block, BlockLabel};
//!
//! let body = Body::builder()
//!     .add_block(
//!         Block::builder("resource")
//!             .add_label("aws_s3_bucket")
//!             .add_label("mybucket")
//!             .add_attribute(("name", "mybucket"))
//!             .add_block(
//!                 Block::builder("logging")
//!                     .add_attribute(("target_bucket", "mylogsbucket"))
//!                     .build()
//!             )
//!             .build()
//!     )
//!     .build();
//!
//! let mut iter = body.attributes();
//!
//! assert_eq!(iter.next(), None);
//!
//! let mut iter = body.blocks();
//!
//! let block = iter.next().unwrap();
//!
//! assert_eq!(block.identifier(), "resource");
//! assert_eq!(
//!     block.labels().first(),
//!     Some(&BlockLabel::from("aws_s3_bucket")),
//! );
//! ```

mod attribute;
mod block;
mod body;
pub(crate) mod de;
pub mod iter;
mod json_spec;
mod ser;
#[cfg(test)]
mod tests;

pub(crate) use self::json_spec::IntoJsonSpec;
pub use self::{
    attribute::Attribute,
    block::{Block, BlockBuilder, BlockLabel},
    body::{Body, BodyBuilder},
};
use crate::Value;
use serde::Deserialize;

/// Represents an HCL structure.
///
/// There are two possible structures that can occur in an HCL [`Body`]: [`Attribute`]s and [`Block`]s.
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Structure {
    /// Represents an HCL attribute.
    Attribute(Attribute),
    /// Represents an HCL block.
    Block(Block),
}

impl Structure {
    /// Returns `true` if the structure represents an [`Attribute`].
    pub fn is_attribute(&self) -> bool {
        self.as_attribute().is_some()
    }

    /// Returns `true` if the structure represents a [`Block`].
    pub fn is_block(&self) -> bool {
        self.as_block().is_some()
    }

    /// Takes ownership of the `Structure` and, if it is an `Attribute`, returns its value,
    /// otherwise `None`.
    pub fn into_attribute(self) -> Option<Attribute> {
        match self {
            Structure::Attribute(attr) => Some(attr),
            Structure::Block(_) => None,
        }
    }

    /// If the `Structure` is an `Attribute`, returns a reference to it, otherwise `None`.
    pub fn as_attribute(&self) -> Option<&Attribute> {
        match self {
            Structure::Attribute(attr) => Some(attr),
            Structure::Block(_) => None,
        }
    }

    /// If the `Structure` is an `Attribute`, returns a mutable reference to it, otherwise `None`.
    pub fn as_attribute_mut(&mut self) -> Option<&mut Attribute> {
        match self {
            Structure::Attribute(attr) => Some(attr),
            Structure::Block(_) => None,
        }
    }

    /// Takes ownership of the `Structure` and, if it is a `Block`, returns its value,
    /// otherwise `None`.
    pub fn into_block(self) -> Option<Block> {
        match self {
            Structure::Block(block) => Some(block),
            Structure::Attribute(_) => None,
        }
    }

    /// If the `Structure` is a `Block`, returns a reference to it, otherwise `None`.
    pub fn as_block(&self) -> Option<&Block> {
        match self {
            Structure::Block(block) => Some(block),
            Structure::Attribute(_) => None,
        }
    }

    /// If the `Structure` is a `Block`, returns a mutable reference to it, otherwise `None`.
    pub fn as_block_mut(&mut self) -> Option<&mut Block> {
        match self {
            Structure::Block(block) => Some(block),
            Structure::Attribute(_) => None,
        }
    }
}

impl From<Structure> for Value {
    fn from(s: Structure) -> Value {
        match s {
            Structure::Attribute(attr) => attr.into(),
            Structure::Block(block) => block.into(),
        }
    }
}

impl From<Attribute> for Structure {
    fn from(attr: Attribute) -> Structure {
        Structure::Attribute(attr)
    }
}

impl From<Block> for Structure {
    fn from(block: Block) -> Structure {
        Structure::Block(block)
    }
}

// Type aliases exposed for backwards compatiblity. Will be removed at one point.
#[allow(missing_docs)]
mod deprecated {
    #[deprecated(since = "0.9.1", note = "use `hcl::Identifier` instead")]
    pub type Identifier = crate::ident::Identifier;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::Conditional` instead")]
    pub type Conditional = crate::expr::Conditional;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::Expression` instead")]
    pub type Expression = crate::expr::Expression;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::Object` instead")]
    pub type Object<K, V> = crate::expr::Object<K, V>;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::ObjectKey` instead")]
    pub type ObjectKey = crate::expr::ObjectKey;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::RawExpression` instead")]
    pub type RawExpression = crate::expr::RawExpression;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::ForExpr` instead")]
    pub type ForExpr = crate::expr::ForExpr;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::FuncCall` instead")]
    pub type FuncCall = crate::expr::FuncCall;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::FuncCallBuilder` instead")]
    pub type FuncCallBuilder = crate::expr::FuncCallBuilder;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::BinaryOp` instead")]
    pub type BinaryOp = crate::expr::BinaryOp;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::BinaryOperator` instead")]
    pub type BinaryOperator = crate::expr::BinaryOperator;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::Operation` instead")]
    pub type Operation = crate::expr::Operation;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::UnaryOp` instead")]
    pub type UnaryOp = crate::expr::UnaryOp;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::UnaryOperator` instead")]
    pub type UnaryOperator = crate::expr::UnaryOperator;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::Heredoc` instead")]
    pub type Heredoc = crate::expr::Heredoc;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::HeredocStripMode` instead")]
    pub type HeredocStripMode = crate::expr::HeredocStripMode;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::TemplateExpr` instead")]
    pub type TemplateExpr = crate::expr::TemplateExpr;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::Traversal` instead")]
    pub type Traversal = crate::expr::Traversal;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::TraversalOperator` instead")]
    pub type TraversalOperator = crate::expr::TraversalOperator;
    #[deprecated(since = "0.9.1", note = "use `hcl::expr::Variable` instead")]
    pub type Variable = crate::expr::Variable;
}

#[doc(hidden)]
pub use self::deprecated::*;
