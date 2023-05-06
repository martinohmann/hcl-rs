//! Types to represent the HCL structural sub-language.

mod attribute;
mod block;
mod body;

pub use self::attribute::Attribute;
pub use self::block::{Block, BlockLabel};
pub use self::body::{
    Attributes, AttributesMut, Blocks, BlocksMut, Body, IntoAttributes, IntoBlocks, IntoIter, Iter,
    IterMut,
};
use crate::repr::{Decor, Decorate, SetSpan, Span};
use std::ops::Range;

/// Represents an HCL structure.
///
/// There are two possible structures that can occur in an HCL [`Body`]: [`Attribute`]s and [`Block`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// If the `Structure` is an `Attribute`, returns it, otherwise `None`.
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

    /// If the `Structure` is a `Block`, returns it, otherwise `None`.
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

    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            Structure::Attribute(attr) => attr.despan(input),
            Structure::Block(block) => block.despan(input),
        }
    }
}

impl From<Attribute> for Structure {
    fn from(value: Attribute) -> Self {
        Structure::Attribute(value)
    }
}

impl From<Block> for Structure {
    fn from(value: Block) -> Self {
        Structure::Block(value)
    }
}

forward_decorate_impl!(Structure => { Attribute, Block });
forward_span_impl!(Structure => { Attribute, Block });
