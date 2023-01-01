//! Types to represent and build HCL blocks.

use super::{Attribute, Body, BodyBuilder, Structure};
use crate::Identifier;
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
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Block {
    /// The block identifier.
    pub identifier: Identifier,
    /// Zero or more block labels.
    pub labels: Vec<BlockLabel>,
    /// Represents the `Block`'s body.
    pub body: Body,
}

impl Block {
    /// Creates a new empty `Block`.
    pub fn new<I>(ident: I) -> Block
    where
        I: Into<Identifier>,
    {
        Block {
            identifier: ident.into(),
            labels: Vec::new(),
            body: Body::default(),
        }
    }

    /// Creates a new [`BlockBuilder`] to start building a new `Block` with the provided
    /// identifier.
    pub fn builder<I>(identifier: I) -> BlockBuilder
    where
        I: Into<Identifier>,
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

impl<I, B> From<(I, B)> for Block
where
    I: Into<Identifier>,
    B: IntoIterator,
    B::Item: Into<Structure>,
{
    fn from((ident, body): (I, B)) -> Block {
        Block {
            identifier: ident.into(),
            labels: Vec::new(),
            body: body.into_iter().collect(),
        }
    }
}

impl<I, L, B> From<(I, L, B)> for Block
where
    I: Into<Identifier>,
    L: IntoIterator,
    L::Item: Into<BlockLabel>,
    B: IntoIterator,
    B::Item: Into<Structure>,
{
    fn from((ident, labels, body): (I, L, B)) -> Block {
        Block {
            identifier: ident.into(),
            labels: labels.into_iter().map(Into::into).collect(),
            body: body.into_iter().collect(),
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
pub enum BlockLabel {
    /// A bare HCL block label.
    Identifier(Identifier),
    /// A quoted string literal.
    String(String),
}

impl BlockLabel {
    /// Creates a new bare `BlockLabel` identifier.
    #[deprecated(since = "0.9.0", note = "use `BlockLabel::from(identifier)` instead")]
    pub fn identifier<I>(identifier: I) -> Self
    where
        I: Into<Identifier>,
    {
        BlockLabel::Identifier(identifier.into())
    }

    /// Creates a new quoted string `BlockLabel`.
    #[deprecated(since = "0.9.0", note = "use `BlockLabel::from(string)` instead")]
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

    /// Borrows the `BlockLabel`'s inner value as a `&str`.
    pub fn as_str(&self) -> &str {
        match self {
            BlockLabel::Identifier(ident) => ident.as_str(),
            BlockLabel::String(string) => string.as_str(),
        }
    }
}

impl<T> From<T> for BlockLabel
where
    T: Into<String>,
{
    fn from(s: T) -> BlockLabel {
        BlockLabel::String(s.into())
    }
}

impl From<Identifier> for BlockLabel {
    fn from(ident: Identifier) -> Self {
        BlockLabel::Identifier(ident)
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
    identifier: Identifier,
    labels: Vec<BlockLabel>,
    body: BodyBuilder,
}

impl BlockBuilder {
    /// Creates a new `BlockBuilder` to start building a new [`Block`] with the provided
    /// identifier.
    pub fn new<I>(identifier: I) -> BlockBuilder
    where
        I: Into<Identifier>,
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
