use super::{Attribute, Body, BodyBuilder, IntoNodeMap, Structure};
use crate::Value;

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub identifier: String,
    pub labels: Vec<BlockLabel>,
    pub body: Body,
}

impl Block {
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

#[derive(Debug, PartialEq, Clone)]
pub enum BlockLabel {
    Identifier(String),
    StringLit(String),
}

impl BlockLabel {
    pub fn identifier<I>(identifier: I) -> Self
    where
        I: Into<String>,
    {
        BlockLabel::Identifier(identifier.into())
    }

    pub fn string_lit<S>(string: S) -> Self
    where
        S: Into<String>,
    {
        BlockLabel::StringLit(string.into())
    }

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

#[derive(Debug)]
pub struct BlockBuilder {
    identifier: String,
    labels: Vec<BlockLabel>,
    body: BodyBuilder,
}

impl BlockBuilder {
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

    pub fn add_label<L>(mut self, label: L) -> BlockBuilder
    where
        L: Into<BlockLabel>,
    {
        self.labels.push(label.into());
        self
    }

    pub fn add_attribute<A>(mut self, attr: A) -> BlockBuilder
    where
        A: Into<Attribute>,
    {
        self.body = self.body.add_attribute(attr);
        self
    }

    pub fn add_block<B>(mut self, block: B) -> BlockBuilder
    where
        B: Into<Block>,
    {
        self.body = self.body.add_block(block);
        self
    }

    pub fn add_structure<S>(mut self, structure: S) -> BlockBuilder
    where
        S: Into<Structure>,
    {
        self.body = self.body.add_structure(structure);
        self
    }

    pub fn build(self) -> Block {
        Block {
            identifier: self.identifier,
            labels: self.labels,
            body: self.body.build(),
        }
    }
}
