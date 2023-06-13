//! Types to represent the HCL structural sub-language.

mod attribute;
mod block;
mod body;

pub use self::attribute::{Attribute, AttributeMut};
pub use self::block::{Block, BlockBuilder, BlockLabel};
pub use self::body::{
    Attributes, AttributesMut, Blocks, BlocksMut, Body, BodyBuilder, IntoAttributes, IntoBlocks,
    IntoIter, Iter, IterMut,
};
use crate::{Decor, Decorate, Span};
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

    /// If the `Structure` is an `Attribute`, returns it.
    ///
    /// # Errors
    ///
    /// An [`Err`](core::result::Result::Err) is returns with the same `Structure` that was passed
    /// in if it is not an `Attribute`.
    pub fn into_attribute(self) -> Result<Attribute, Structure> {
        match self {
            Structure::Attribute(attr) => Ok(attr),
            Structure::Block(_) => Err(self),
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

    /// If the `Structure` is a `Block`, returns it.
    ///
    /// # Errors
    ///
    /// An [`Err`](core::result::Result::Err) is returns with the same `Structure` that was passed
    /// in if it is not a `Block`.
    pub fn into_block(self) -> Result<Block, Structure> {
        match self {
            Structure::Block(block) => Ok(block),
            Structure::Attribute(_) => Err(self),
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

/// Allows mutable access to a structure, except for attribute keys which are immutable.
///
/// This type wraps the structure in the iterator returned by
/// [`Body::iter_mut`](crate::structure::Body::iter_mut).
pub struct StructureMut<'a> {
    structure: &'a mut Structure,
}

impl<'a> StructureMut<'a> {
    pub(crate) fn new(structure: &'a mut Structure) -> StructureMut<'a> {
        StructureMut { structure }
    }

    /// Returns `true` if the structure represents an [`Attribute`].
    pub fn is_attribute(&self) -> bool {
        self.as_attribute().is_some()
    }

    /// Returns `true` if the structure represents a [`Block`].
    pub fn is_block(&self) -> bool {
        self.as_block().is_some()
    }

    /// If the `Structure` is an `Attribute`, returns a reference to it, otherwise `None`.
    pub fn as_attribute(&self) -> Option<&Attribute> {
        self.structure.as_attribute()
    }

    /// If the `Structure` is an `Attribute`, returns a mutable reference to it, otherwise `None`.
    pub fn as_attribute_mut(&mut self) -> Option<AttributeMut<'_>> {
        self.structure.as_attribute_mut().map(AttributeMut::new)
    }

    /// If the `Structure` is a `Block`, returns a reference to it, otherwise `None`.
    pub fn as_block(&self) -> Option<&Block> {
        self.structure.as_block()
    }

    /// If the `Structure` is a `Block`, returns a mutable reference to it, otherwise `None`.
    pub fn as_block_mut(&mut self) -> Option<&mut Block> {
        self.structure.as_block_mut()
    }
}

impl<'a> Decorate for StructureMut<'a> {
    fn decor(&self) -> &Decor {
        self.structure.decor()
    }

    fn decor_mut(&mut self) -> &mut Decor {
        self.structure.decor_mut()
    }
}

impl<'a> Span for StructureMut<'a> {
    fn span(&self) -> Option<Range<usize>> {
        self.structure.span()
    }
}
