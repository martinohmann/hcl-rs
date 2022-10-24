//! Types to represent HCL structures.
//!
//! The main types in this module are:
//!
//! - [`Attribute`]: represent an HCL attribute
//! - [`Block`]: represent an HCL block
//! - [`BlockBuilder`]: provides functionality for building `Block`s
//! - [`Body`]: represent the body of an HCL configuration or block
//! - [`BodyBuilder`]: provides functionality for building `Body`s
//! - [`Expression`]: represent the value of an HCL attribute
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
//!     Some(&BlockLabel::string("aws_s3_bucket")),
//! );
//! ```

mod attribute;
mod block;
mod body;
mod conditional;
pub(crate) mod de;
mod expression;
mod for_expr;
mod func_call;
mod operation;
pub(crate) mod ser;
mod template_expr;
#[cfg(test)]
mod tests;
mod traversal;

pub use self::{
    attribute::Attribute,
    block::{Block, BlockBuilder, BlockLabel},
    body::{Body, BodyBuilder},
    conditional::Conditional,
    expression::{Expression, Object, ObjectKey, RawExpression},
    for_expr::ForExpr,
    func_call::{FuncCall, FuncCallBuilder},
    operation::{BinaryOp, BinaryOperator, Operation, UnaryOp, UnaryOperator},
    template_expr::{Heredoc, HeredocStripMode, TemplateExpr},
    traversal::{Traversal, TraversalOperator},
};
use crate::{Error, Map, Result, Value};
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, Cow};
use std::fmt;
use std::ops;
use std::str::FromStr;

/// Represents an HCL identifier.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename = "$hcl::identifier")]
pub struct Identifier(String);

impl Identifier {
    /// Creates a new `Identifier`.
    ///
    /// If `ident` contains characters that are not allowed in HCL identifiers will be sanitized
    /// according to the following rules:
    ///
    /// - An empty `ident` results in an identifier containing a single underscore.
    /// - Invalid characters in `ident` will be replaced with underscores.
    /// - If `ident` starts with a character that is invalid in the first position but would be
    ///   valid in the rest of an HCL identifier it is prefixed with an underscore.
    ///
    /// See [`Identifier::from_str`][Identifier::from_str] for a fallible alternative to this
    /// function if you prefer rejecting invalid identifiers instead of sanitizing them.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl::Identifier;
    /// assert_eq!(Identifier::new("some_ident").as_str(), "some_ident");
    /// assert_eq!(Identifier::new("").as_str(), "_");
    /// assert_eq!(Identifier::new("1two3").as_str(), "_1two3");
    /// assert_eq!(Identifier::new("with whitespace").as_str(), "with_whitespace");
    /// ```
    pub fn new<T>(ident: T) -> Self
    where
        T: AsRef<str>,
    {
        let input = ident.as_ref();

        if input.is_empty() {
            return Identifier::new_unchecked('_');
        }

        let mut ident = String::with_capacity(input.len());

        for (i, ch) in input.chars().enumerate() {
            if i == 0 && is_id_start(ch) {
                ident.push(ch);
            } else if is_id_continue(ch) {
                if i == 0 {
                    ident.push('_');
                }
                ident.push(ch);
            } else {
                ident.push('_');
            }
        }

        Identifier::new_unchecked(ident)
    }

    /// Creates a new `Identifier` without checking if it is valid in HCL.
    ///
    /// It is the caller's responsibility to ensure that the identifier is valid.
    ///
    /// # Safety
    ///
    /// This function is not marked as unsafe because it does not cause undefined behaviour.
    /// However, attempting to serialize an invalid identifier to HCL will produce invalid output.
    pub fn new_unchecked<T>(ident: T) -> Self
    where
        T: Into<String>,
    {
        Identifier(ident.into())
    }

    /// Consumes `self` and returns the `Identifier` as a `String`.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns the `Identifier` as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Identifier {
    type Err = Error;

    /// Creates a new `Identifier` from a `str`.
    ///
    /// See [`Identifier::new`][Identifier::new] for an infallible alternative to this function.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use hcl::Identifier;
    /// assert_eq!(Identifier::from_str("some_ident").unwrap(), Identifier::new("some_ident"));
    /// assert!(Identifier::from_str("").is_err());
    /// assert!(Identifier::from_str("1two3").is_err());
    /// assert!(Identifier::from_str("with whitespace").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// If `s` contains characters that are not allowed in HCL identifiers an error will be
    /// returned.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error::InvalidIdentifier(s.to_owned()));
        }

        let mut chars = s.chars();
        let start = chars.next().unwrap();

        if !is_id_start(start) || !chars.all(is_id_continue) {
            return Err(Error::InvalidIdentifier(s.to_owned()));
        }

        Ok(Identifier::new_unchecked(s))
    }
}

#[inline]
fn is_id_start(ch: char) -> bool {
    ch == '_' || unicode_ident::is_xid_start(ch)
}

#[inline]
fn is_id_continue(ch: char) -> bool {
    ch == '-' || unicode_ident::is_xid_continue(ch)
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Identifier::new(s)
    }
}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Identifier::new(s)
    }
}

impl<'a> From<Cow<'a, str>> for Identifier {
    fn from(s: Cow<'a, str>) -> Self {
        Identifier::new(s)
    }
}

impl From<Identifier> for String {
    fn from(ident: Identifier) -> Self {
        ident.into_inner()
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl ops::Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

/// Represents an HCL structure.
///
/// There are two possible structures that can occur in an HCL [`Body`]: [`Attribute`]s and [`Block`]s.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename = "$hcl::structure")]
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
