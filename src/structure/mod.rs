//! Maps the structure of a HCL configuration to rust data types.

mod de;
mod from;
mod ser;

use crate::value::Value;
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;

/// The body of a HCL config file or block.
#[derive(Debug, PartialEq, Clone)]
pub struct Body {
    inner: Vec<Structure>,
}

impl Body {
    /// Returns the number of structures in the `Body`.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the `Body` does not contain any structures.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns an iterator over the structures in the `Body`.
    pub fn iter(&self) -> StructureIter {
        StructureIter {
            inner: self.inner.iter(),
        }
    }

    /// Returns an iterator that allows modifying each `Structure` in the `Body`.
    pub fn iter_mut(&mut self) -> StructureIterMut {
        StructureIterMut {
            inner: self.inner.iter_mut(),
        }
    }

    /// Returns an iterator over all `Attribute` structures in the `Body`.
    pub fn attributes(&self) -> AttributeIter {
        AttributeIter {
            inner: self.inner.iter(),
        }
    }

    /// Returns an iterator that allows modifying each `Attribute` structure in the `Body`.
    pub fn attributes_mut(&mut self) -> AttributeIterMut {
        AttributeIterMut {
            inner: self.inner.iter_mut(),
        }
    }

    /// Returns an iterator over all `Block` structures in the `Body`.
    pub fn blocks(&self) -> BlockIter {
        BlockIter {
            inner: self.inner.iter(),
        }
    }

    /// Returns an iterator that allows modifying each `Block` structure in the `Body`.
    pub fn blocks_mut(&mut self) -> BlockIterMut {
        BlockIterMut {
            inner: self.inner.iter_mut(),
        }
    }

    /// Returns true if the `Body` contains any `Attribute` structures.
    pub fn has_attributes(&self) -> bool {
        self.iter().any(|s| s.is_attribute())
    }

    /// Returns true if the `Body` contains any `Block` structures.
    pub fn has_blocks(&self) -> bool {
        self.iter().any(|s| s.is_block())
    }
}

impl FromIterator<Structure> for Body {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Structure>,
    {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for Body {
    type Item = Structure;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

/// Represents a HCL structure.
#[derive(Debug, PartialEq, Clone)]
pub enum Structure {
    /// An Attribute is a key-value pair where the key is a string identifier. The value can be a
    /// literal value or complex expression.
    Attribute(Attribute),
    /// A nested block which has an identifier, zero or more keys and a body.
    Block(Block),
}

impl Structure {
    /// If the `Structure` is an Attribute, returns the associated `Attribute`. Returns None
    /// otherwise.
    pub fn as_attribute(&self) -> Option<&Attribute> {
        match self {
            Self::Attribute(attr) => Some(attr),
            Self::Block(_) => None,
        }
    }

    /// If the `Structure` is an Attribute, returns the associated mutable `Attribute`. Returns
    /// None otherwise.
    pub fn as_attribute_mut(&mut self) -> Option<&mut Attribute> {
        match self {
            Self::Attribute(attr) => Some(attr),
            Self::Block(_) => None,
        }
    }

    /// If the `Structure` is a Block, returns the associated `Block`. Returns None
    /// otherwise.
    pub fn as_block(&self) -> Option<&Block> {
        match self {
            Self::Block(block) => Some(block),
            Self::Attribute(_) => None,
        }
    }

    /// If the `Structure` is a Block, returns the associated mutable `Block`. Returns None
    /// otherwise.
    pub fn as_block_mut(&mut self) -> Option<&mut Block> {
        match self {
            Self::Block(block) => Some(block),
            Self::Attribute(_) => None,
        }
    }

    /// Returns true if the `Structure` is an Attribute. Returns false otherwise.
    ///
    /// For any Structure on which `is_attribute` returns true, `as_attribute` and
    /// `as_attribute_mut` are guaranteed to return the `Attribute`.
    pub fn is_attribute(&self) -> bool {
        self.as_attribute().is_some()
    }

    /// Returns true if the `Structure` is a Block. Returns false otherwise.
    ///
    /// For any Structure on which `is_block` returns true, `as_block` and `as_block_mut` are
    /// guaranteed to return the `Block`.
    pub fn is_block(&self) -> bool {
        self.as_block().is_some()
    }
}

/// Represents a HCL attribute.
#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    key: String,
    value: Value,
}

impl Attribute {
    /// Create a new `Attribute` from a `&str` key and a `Value`.
    pub fn new(key: &str, value: Value) -> Self {
        Self {
            key: key.to_owned(),
            value,
        }
    }

    /// Returns the attribute key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the attribute value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Returns a mutable reference to the attribute value.
    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    /// Creates a new `Attribute` from a new key and the existing value.
    pub fn with_key(self, key: &str) -> Self {
        Self {
            key: key.to_owned(),
            value: self.value,
        }
    }

    /// Creates a new `Attribute` from a new value and the existing key.
    pub fn with_value(self, value: Value) -> Self {
        Self {
            key: self.key,
            value,
        }
    }
}

/// Represents a HCL block.
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    ident: String,
    keys: Vec<String>,
    body: Body,
}

impl Block {
    /// Creates a new `Block` from a `&str` identifier, an optional list of block keys and a block
    /// body.
    pub fn new<K, B>(ident: &str, keys: K, body: B) -> Self
    where
        K: IntoIterator<Item = String>,
        B: IntoIterator<Item = Structure>,
    {
        Self {
            ident: ident.to_owned(),
            keys: keys.into_iter().collect(),
            body: body.into_iter().collect(),
        }
    }

    /// Returns the block identifier.
    pub fn ident(&self) -> &str {
        &self.ident
    }

    /// Returns the block keys.
    pub fn keys(&self) -> &Vec<String> {
        &self.keys
    }

    /// Returns a mutable reference to the block keys.
    pub fn keys_mut(&mut self) -> &mut Vec<String> {
        &mut self.keys
    }

    /// Returns the block body.
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Returns a mutable reference to the block body.
    pub fn body_mut(&mut self) -> &mut Body {
        &mut self.body
    }

    /// Creates a new `Block` from new block identifier and the existing block keys and block
    /// body.
    pub fn with_ident(self, ident: &str) -> Self {
        Self {
            ident: ident.to_owned(),
            keys: self.keys,
            body: self.body,
        }
    }

    /// Creates a new `Block` from new block keys and the existing block identifier and block
    /// body.
    pub fn with_keys<K>(self, keys: K) -> Self
    where
        K: IntoIterator<Item = String>,
    {
        Self {
            ident: self.ident,
            keys: keys.into_iter().collect(),
            body: self.body,
        }
    }

    /// Creates a new `Block` from a new block body and the existing block identifier and block
    /// keys.
    pub fn with_body<B>(self, body: B) -> Self
    where
        B: IntoIterator<Item = Structure>,
    {
        Self {
            ident: self.ident,
            keys: self.keys,
            body: body.into_iter().collect(),
        }
    }
}

/// An `Iterator` over HCL structures.
pub struct StructureIter<'a> {
    inner: Iter<'a, Structure>,
}

impl<'a> Iterator for StructureIter<'a> {
    type Item = &'a Structure;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// A mutable `Iterator` over HCL structures.
pub struct StructureIterMut<'a> {
    inner: IterMut<'a, Structure>,
}

impl<'a> Iterator for StructureIterMut<'a> {
    type Item = &'a mut Structure;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// An `Iterator` over `Attribute` structures.
pub struct AttributeIter<'a> {
    inner: Iter<'a, Structure>,
}

impl<'a> Iterator for AttributeIter<'a> {
    type Item = &'a Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            if let Some(attr) = structure.as_attribute() {
                return Some(attr);
            }
        }

        None
    }
}

/// A mutable `Iterator` over `Attribute` structures.
pub struct AttributeIterMut<'a> {
    inner: IterMut<'a, Structure>,
}

impl<'a> Iterator for AttributeIterMut<'a> {
    type Item = &'a mut Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            if let Some(attr) = structure.as_attribute_mut() {
                return Some(attr);
            }
        }

        None
    }
}

/// An `Iterator` over `Block` structures.
pub struct BlockIter<'a> {
    inner: Iter<'a, Structure>,
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            if let Some(block) = structure.as_block() {
                return Some(block);
            }
        }

        None
    }
}

/// A mutable `Iterator` over `Block` structures.
pub struct BlockIterMut<'a> {
    inner: IterMut<'a, Structure>,
}

impl<'a> Iterator for BlockIterMut<'a> {
    type Item = &'a mut Block;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            if let Some(block) = structure.as_block_mut() {
                return Some(block);
            }
        }

        None
    }
}
