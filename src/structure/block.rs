use super::{Attribute, Body, BodyBuilder, IntoNodeMap, Structure};
use crate::Value;

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
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub identifier: String,
    pub labels: Vec<BlockLabel>,
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
}

impl From<Block> for Value {
    fn from(block: Block) -> Value {
        Value::from_iter(block.into_node_map())
    }
}

impl<I, B> From<(I, B)> for Block
where
    I: Into<String>,
    B: IntoIterator,
    B::Item: Into<Structure>,
{
    fn from(pair: (I, B)) -> Block {
        Block {
            identifier: pair.0.into(),
            labels: Vec::new(),
            body: pair.1.into_iter().collect(),
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
#[derive(Debug, PartialEq, Clone)]
pub enum BlockLabel {
    Identifier(String),
    StringLit(String),
}

impl BlockLabel {
    /// Creates a new bare `BlockLabel` identifier.
    pub fn identifier<I>(identifier: I) -> Self
    where
        I: Into<String>,
    {
        BlockLabel::Identifier(identifier.into())
    }

    /// Creates a new quoted string `BlockLabel`.
    pub fn string_lit<S>(string: S) -> Self
    where
        S: Into<String>,
    {
        BlockLabel::StringLit(string.into())
    }

    /// Consumes `self` and returns the `String` wrapped by the `BlockLabel`.
    ///
    /// Beware that after calling `.into_inner()` it is not possible anymore to tell whether the
    /// `String` resembles a quoted string or bare identifer.
    pub fn into_inner(self) -> String {
        match self {
            BlockLabel::Identifier(ident) => ident,
            BlockLabel::StringLit(string) => string,
        }
    }
}

impl<T> From<T> for BlockLabel
where
    T: Into<String>,
{
    fn from(v: T) -> BlockLabel {
        BlockLabel::string_lit(v)
    }
}

/// `BlockBuilder` builds a HCL [`Block`].
///
/// The builder allows build the `Block` by adding labels, attributes and other nested blocks via
/// chained method calls. A call to [`.build()`](BlockBuilder::build) produces the final `Block`.
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

    /// Consumes `self` and builds the [`Block`] from the items added via the builder methods.
    pub fn build(self) -> Block {
        Block {
            identifier: self.identifier,
            labels: self.labels,
            body: self.body.build(),
        }
    }
}
