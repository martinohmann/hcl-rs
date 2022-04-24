//! Types to represent HCL structures.
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
//!     Some(&BlockLabel::StringLit("aws_s3_bucket".into())),
//! );
//! ```

pub mod attribute;
pub mod block;
pub mod body;
pub mod expression;
mod ser;

pub use self::{
    attribute::Attribute,
    block::{Block, BlockBuilder, BlockLabel},
    body::{Body, BodyBuilder},
    expression::{Expression, Object, ObjectKey, RawExpression},
};
use crate::{Map, Value};

/// Represents an HCL structure.
///
/// There are two possible structures that can occur in an HCL [`Body`]: [`Attribute`]s and [`Block`]s.
#[derive(Debug, PartialEq, Clone)]
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

// A trait to convert an HCL structure into a map of nodes.
//
// This is used internally by the `Body` and `Block` types to convert into a `Value`.
//
// The detour over a map of nodes is necessary as HCL blocks with the same identifier and labels
// need to be merged so that the resulting `Value` conforms to the [HCL JSON
// specification](hcl-json-spec).
//
// [hcl-json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md#blocks
trait IntoNodeMap {
    fn into_node_map(self) -> Map<String, Node>;
}

impl IntoNodeMap for Body {
    fn into_node_map(self) -> Map<String, Node> {
        self.into_iter().fold(Map::new(), |mut map, structure| {
            match structure {
                Structure::Attribute(attr) => {
                    map.insert(attr.key, Node::Value(attr.expr.into()));
                }
                Structure::Block(block) => {
                    block
                        .into_node_map()
                        .into_iter()
                        .for_each(|(key, mut node)| {
                            map.entry(key)
                                .and_modify(|entry| entry.deep_merge(&mut node))
                                .or_insert(node);
                        });
                }
            };

            map
        })
    }
}

impl IntoNodeMap for Block {
    fn into_node_map(self) -> Map<String, Node> {
        let mut labels = self.labels.into_iter();

        let node = match labels.next() {
            Some(label) => {
                let block = Block {
                    identifier: label.into_inner(),
                    labels: labels.collect(),
                    body: self.body,
                };

                Node::Block(block.into_node_map())
            }
            None => Node::BlockInner(vec![self.body]),
        };

        Map::from_iter(std::iter::once((self.identifier, node)))
    }
}

enum Node {
    Empty,
    Block(Map<String, Node>),
    BlockInner(Vec<Body>),
    Value(Value),
}

impl From<Node> for Value {
    fn from(node: Node) -> Value {
        match node {
            Node::Empty => Value::Null,
            Node::Block(map) => Value::from_iter(map),
            Node::BlockInner(mut vec) => {
                // Flatten as per the [HCL JSON spec](json-spec).
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
            Node::Value(value) => value,
        }
    }
}

impl Node {
    fn take(&mut self) -> Node {
        std::mem::replace(self, Node::Empty)
    }

    fn deep_merge(&mut self, other: &mut Node) {
        match (self, other) {
            (Node::Block(lhs), Node::Block(rhs)) => {
                rhs.iter_mut().for_each(|(key, node)| {
                    lhs.entry(key.to_string())
                        .and_modify(|lhs| lhs.deep_merge(node))
                        .or_insert_with(|| node.take());
                });
            }
            (Node::BlockInner(lhs), Node::BlockInner(rhs)) => {
                lhs.append(rhs);
            }
            (lhs, rhs) => *lhs = rhs.take(),
        }
    }
}

#[doc(hidden)]
pub(crate) mod private {
    pub const ATTRIBUTE_NAME: &str = "$__hcl_private_Attribute";
    pub const BLOCK_NAME: &str = "$__hcl_private_Block";
    pub const IDENT_NAME: &str = "$__hcl_private_Identifier";
    pub const RAW_EXPRESSION_NAME: &str = "$__hcl_private_RawExpression";

    pub const BLOCK_BODY_FIELD: &str = "$__hcl_private_block_body";
    pub const BLOCK_LABELS_FIELD: &str = "$__hcl_private_block_labels";
    pub const EXPRESSION_FIELD: &str = "$__hcl_private_expression";
    pub const IDENT_FIELD: &str = "$__hcl_private_identifier";
    pub const RAW_EXPRESSION_FIELD: &str = "$__hcl_private_raw_expression";
}
