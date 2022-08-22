//! Types to represent and build HCL blocks.

use super::{Attribute, Body, BodyBuilder, Identifier, IntoNodeMap, Structure};
use crate::Value;
use serde::{Deserialize, Serialize};

/// Represents an HCL block which consists of a block identifier, zero or more block labels and a
/// block body.
///
/// In HCL syntax this is represented as:
///
/// ```hcl
/// block_identifier "block_label1" "block_label2" {
///   body
/// }
/// ```
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename = "$hcl::block")]
pub struct Block {
    /// The block identifier.
    pub identifier: String,
    /// Zero or more block labels.
    pub labels: Vec<BlockLabel>,
    /// Represents the `Block`'s body.
    pub body: Body,
}

impl Block {
    /// Creates a new `Block` from a block identifier, block labels and a block body.
    pub fn new<I, L, B>(identifier: I, labels: L, body: B) -> Block
    where
        I: Into<String>,
        L: IntoIterator,
        L::Item: Into<BlockLabel>,
        B: IntoIterator,
        B::Item: Into<Structure>,
    {
        Block {
            identifier: identifier.into(),
            labels: labels.into_iter().map(Into::into).collect(),
            body: body.into_iter().collect(),
        }
    }

    /// Creates a new [`BlockBuilder`] to start building a new `Block` with the provided
    /// identifier.
    pub fn builder<I>(identifier: I) -> BlockBuilder
    where
        I: Into<String>,
    {
        BlockBuilder::new(identifier)
    }

    /// Returns a reference to the block's identifier.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Returns a reference to the block's labels.
    pub fn labels(&self) -> &[BlockLabel] {
        &self.labels
    }

    /// Returns a reference to the block's body.
    pub fn body(&self) -> &Body {
        &self.body
    }
}

impl From<Block> for Value {
    fn from(block: Block) -> Value {
        Value::from_iter(block.into_node_map())
    }
}

impl<I, B> From<(I, B)> for Block
where
    I: Into<String>,
    B: Into<Body>,
{
    fn from(pair: (I, B)) -> Block {
        Block {
            identifier: pair.0.into(),
            labels: Vec::new(),
            body: pair.1.into(),
        }
    }
}

/// Represents an HCL block label.
///
/// In HCL syntax this can be represented either as a quoted string literal...
///
/// ```hcl
/// block_identifier "block_label1" {
///   body
/// }
/// ```
///
/// ...or as a bare identifier:
///
/// ```hcl
/// block_identifier block_label1 {
///   body
/// }
/// ```
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename = "$hcl::block_label")]
pub enum BlockLabel {
    /// A bare HCL block label.
    Identifier(Identifier),
    /// A quoted string literal.
    String(String),
}

impl BlockLabel {
    /// Creates a new bare `BlockLabel` identifier.
    pub fn identifier<I>(identifier: I) -> Self
    where
        I: Into<Identifier>,
    {
        BlockLabel::Identifier(identifier.into())
    }

    /// Creates a new quoted string `BlockLabel`.
    pub fn string<S>(string: S) -> Self
    where
        S: Into<String>,
    {
        BlockLabel::String(string.into())
    }

    /// Consumes `self` and returns the `String` wrapped by the `BlockLabel`.
    ///
    /// Beware that after calling `.into_inner()` it is not possible anymore to tell whether the
    /// `String` resembles a quoted string or bare identifer.
    pub fn into_inner(self) -> String {
        match self {
            BlockLabel::Identifier(ident) => ident.into_inner(),
            BlockLabel::String(string) => string,
        }
    }
}

impl<T> From<T> for BlockLabel
where
    T: Into<String>,
{
    fn from(v: T) -> BlockLabel {
        BlockLabel::string(v)
    }
}

/// `BlockBuilder` builds an HCL [`Block`].
///
/// The builder allows to build the `Block` by adding labels, attributes and other nested blocks
/// via chained method calls. A call to [`.build()`](BlockBuilder::build) produces the final
/// `Block`.
///
/// ## Example
///
/// ```
/// use hcl::Block;
///
/// let block = Block::builder("resource")
///     .add_label("aws_s3_bucket")
///     .add_label("mybucket")
///     .add_attribute(("name", "mybucket"))
///     .add_block(
///         Block::builder("logging")
///             .add_attribute(("target_bucket", "mylogsbucket"))
///             .build()
///     )
///     .build();
/// ```
#[derive(Debug)]
pub struct BlockBuilder {
    identifier: String,
    labels: Vec<BlockLabel>,
    body: BodyBuilder,
}

impl BlockBuilder {
    /// Creates a new `BlockBuilder` to start building a new [`Block`] with the provided
    /// identifier.
    pub fn new<I>(identifier: I) -> BlockBuilder
    where
        I: Into<String>,
    {
        BlockBuilder {
            identifier: identifier.into(),
            labels: Vec::new(),
            body: Body::builder(),
        }
    }

    /// Adds a `BlockLabel`.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    pub fn add_label<L>(mut self, label: L) -> BlockBuilder
    where
        L: Into<BlockLabel>,
    {
        self.labels.push(label.into());
        self
    }

    /// Adds `BlockLabel`s from an iterator.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    pub fn add_labels<I>(mut self, iter: I) -> BlockBuilder
    where
        I: IntoIterator,
        I::Item: Into<BlockLabel>,
    {
        self.labels.extend(iter.into_iter().map(Into::into));
        self
    }

    /// Adds an `Attribute` to the block body.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    pub fn add_attribute<A>(mut self, attr: A) -> BlockBuilder
    where
        A: Into<Attribute>,
    {
        self.body = self.body.add_attribute(attr);
        self
    }

    /// Adds `Attribute`s to the block body from an iterator.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    pub fn add_attributes<I>(mut self, iter: I) -> BlockBuilder
    where
        I: IntoIterator,
        I::Item: Into<Attribute>,
    {
        self.body = self.body.add_attributes(iter.into_iter().map(Into::into));
        self
    }

    /// Adds another `Block` to the block body.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    pub fn add_block<B>(mut self, block: B) -> BlockBuilder
    where
        B: Into<Block>,
    {
        self.body = self.body.add_block(block);
        self
    }

    /// Adds `Block`s to the block body from an iterator.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    pub fn add_blocks<I>(mut self, iter: I) -> BlockBuilder
    where
        I: IntoIterator,
        I::Item: Into<Block>,
    {
        self.body = self.body.add_blocks(iter.into_iter().map(Into::into));
        self
    }

    /// Adds a `Structure` to the block body.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    pub fn add_structure<S>(mut self, structure: S) -> BlockBuilder
    where
        S: Into<Structure>,
    {
        self.body = self.body.add_structure(structure);
        self
    }

    /// Adds `Structure`s to the block body from an iterator.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    pub fn add_structures<I>(mut self, iter: I) -> BlockBuilder
    where
        I: IntoIterator,
        I::Item: Into<Structure>,
    {
        self.body = self.body.add_structures(iter.into_iter().map(Into::into));
        self
    }

    /// Consumes `self` and builds the [`Block`] from the items added via the builder methods.
    pub fn build(self) -> Block {
        Block {
            identifier: self.identifier,
            labels: self.labels,
            body: self.body.build(),
        }
    }
}
