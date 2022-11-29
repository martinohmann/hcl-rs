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
mod ser;
#[cfg(test)]
mod tests;

pub use self::{
    attribute::Attribute,
    block::{Block, BlockBuilder, BlockLabel},
    body::{Body, BodyBuilder},
};
use crate::{expr, ident, Map, Value};
use indexmap::map::Entry;
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

    /// If the `Structure` is an Attribute, returns a reference to the `Attribute`. Returns None
    /// otherwise.
    pub fn as_attribute(&self) -> Option<&Attribute> {
        match self {
            Structure::Attribute(attr) => Some(attr),
            Structure::Block(_) => None,
        }
    }

    /// If the `Structure` is an Attribute, returns a mutable reference to the `Attribute`. Returns
    /// None otherwise.
    pub fn as_attribute_mut(&mut self) -> Option<&mut Attribute> {
        match self {
            Structure::Attribute(attr) => Some(attr),
            Structure::Block(_) => None,
        }
    }

    /// If the `Structure` is a Block, returns a reference to the `Block`. Returns None otherwise.
    pub fn as_block(&self) -> Option<&Block> {
        match self {
            Structure::Block(block) => Some(block),
            Structure::Attribute(_) => None,
        }
    }

    /// If the `Structure` is a Block, returns a mutable reference to the `Block`. Returns None
    /// otherwise.
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

// A trait to convert an HCL structure into its [JSON representation][json-spec].
//
// This is used internally by the `Body` and `Block` types to convert into a `Value`.
//
// The detour over a map of nodes is necessary as HCL blocks with the same identifier and labels
// need to be merged so that the resulting `Value` conforms to the [HCL JSON
// specification][json-spec].
//
// [json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md#blocks
trait IntoJsonSpec {
    fn into_json_spec(self) -> Map<String, JsonNode>;
}

impl IntoJsonSpec for Body {
    fn into_json_spec(self) -> Map<String, JsonNode> {
        self.into_iter().fold(Map::new(), |mut map, structure| {
            match structure {
                Structure::Attribute(attr) => {
                    map.insert(attr.key.into_inner(), JsonNode::Expr(attr.expr));
                }
                Structure::Block(block) => {
                    block
                        .into_json_spec()
                        .into_iter()
                        .for_each(|(key, node)| node.deep_merge_into(&mut map, key));
                }
            };

            map
        })
    }
}

impl IntoJsonSpec for Block {
    fn into_json_spec(self) -> Map<String, JsonNode> {
        let mut labels = self.labels.into_iter();

        let node = match labels.next() {
            Some(label) => {
                let block = Block {
                    identifier: ident::Identifier::unchecked(label.into_inner()),
                    labels: labels.collect(),
                    body: self.body,
                };

                JsonNode::Map(block.into_json_spec())
            }
            None => JsonNode::Body(vec![self.body]),
        };

        std::iter::once((self.identifier.into_inner(), node)).collect()
    }
}

enum JsonNode {
    Map(Map<String, JsonNode>),
    Body(Vec<Body>),
    Expr(expr::Expression),
}

impl From<JsonNode> for Value {
    fn from(node: JsonNode) -> Value {
        match node {
            JsonNode::Map(map) => Value::from_iter(map),
            JsonNode::Body(mut vec) => {
                // Flatten as per the [HCL JSON spec][json-spec].
                //
                // > After any labelling levels, the next nested value is either a JSON
                // > object representing a single block body, or a JSON array of JSON
                // > objects that each represent a single block body.
                //
                // [json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md#blocks
                if vec.len() == 1 {
                    vec.remove(0).into()
                } else {
                    vec.into()
                }
            }
            JsonNode::Expr(expr) => expr.into(),
        }
    }
}

impl JsonNode {
    fn deep_merge_into(self, map: &mut Map<String, JsonNode>, key: String) {
        match map.entry(key) {
            Entry::Occupied(o) => o.into_mut().deep_merge(self),
            Entry::Vacant(v) => {
                v.insert(self);
            }
        }
    }

    fn deep_merge(&mut self, other: JsonNode) {
        match (self, other) {
            (JsonNode::Map(lhs), JsonNode::Map(rhs)) => {
                rhs.into_iter()
                    .for_each(|(key, node)| node.deep_merge_into(lhs, key));
            }
            (JsonNode::Body(lhs), JsonNode::Body(mut rhs)) => {
                lhs.append(&mut rhs);
            }
            (lhs, rhs) => *lhs = rhs,
        }
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
