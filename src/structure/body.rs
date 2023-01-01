//! Types to represent and build HCL body structures.

use super::iter::{
    Attributes, AttributesMut, Blocks, BlocksMut, IntoAttributes, IntoBlocks, Iter, IterMut,
};
use super::ser::BodySerializer;
use super::{Attribute, Block, Structure};
use crate::ser::with_internal_serialization;
use crate::Result;
use serde::{Deserialize, Serialize};

/// Represents an HCL config file body.
///
/// A `Body` consists of zero or more [`Attribute`] and [`Block`] HCL structures.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Default, Clone)]
#[serde(rename = "$hcl::Body")]
pub struct Body(pub Vec<Structure>);

impl Body {
    #[doc(hidden)]
    pub fn from_serializable<T>(value: &T) -> Result<Body>
    where
        T: ?Sized + Serialize,
    {
        with_internal_serialization(|| value.serialize(BodySerializer))
    }

    /// Consumes `self` and returns the wrapped `Vec<Structure>`.
    pub fn into_inner(self) -> Vec<Structure> {
        self.0
    }

    /// Creates a new [`BodyBuilder`] to start building a new `Body`.
    pub fn builder() -> BodyBuilder {
        BodyBuilder::default()
    }

    /// An iterator visiting all structures within the `Body`. The iterator element type is `&'a
    /// Structure`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl::{Attribute, Body};
    ///
    /// let body = Body::from([
    ///     Attribute::new("a", 1),
    ///     Attribute::new("b", 2),
    ///     Attribute::new("c", 3),
    /// ]);
    ///
    /// for structure in body.iter() {
    ///     println!("{structure:?}");
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }

    /// An iterator visiting all structures within the `Body`. The iterator element type is `&'a
    /// mut Structure`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl::{Attribute, Block, Body, Identifier, Structure};
    ///
    /// let mut body = Body::from([
    ///     Structure::Attribute(Attribute::new("a", 1)),
    ///     Structure::Block(Block::new("b")),
    ///     Structure::Attribute(Attribute::new("c", 3)),
    /// ]);
    ///
    /// // Update all attribute keys and block identifiers
    /// for structure in body.iter_mut() {
    ///     match structure {
    ///         Structure::Attribute(attr) => {
    ///             attr.key = Identifier::new(format!("attr_{}", attr.key)).unwrap();
    ///         }
    ///         Structure::Block(block) => {
    ///             block.identifier = Identifier::new(format!("block_{}", block.identifier)).unwrap();
    ///         }
    ///     }
    /// }
    ///
    /// assert_eq!(body.into_inner(), [
    ///     Structure::Attribute(Attribute::new("attr_a", 1)),
    ///     Structure::Block(Block::new("block_b")),
    ///     Structure::Attribute(Attribute::new("attr_c", 3)),
    /// ]);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut::new(self)
    }

    /// An iterator visiting all attributes within the `Body`. The iterator element type is `&'a
    /// Attribute`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl::{Attribute, Block, Body, Structure};
    ///
    /// let body = Body::from([
    ///     Structure::Attribute(Attribute::new("a", 1)),
    ///     Structure::Block(Block::new("b")),
    ///     Structure::Attribute(Attribute::new("c", 3)),
    /// ]);
    ///
    /// let vec: Vec<&Attribute> = body.attributes().collect();
    /// assert_eq!(vec, [&Attribute::new("a", 1), &Attribute::new("c", 3)]);
    /// ```
    pub fn attributes(&self) -> Attributes<'_> {
        Attributes::new(self)
    }

    /// An iterator visiting all attributes within the `Body`. The iterator element type is `&'a
    /// mut Attribute`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl::{Attribute, Block, Body, Identifier, Structure};
    ///
    /// let mut body = Body::from([
    ///     Structure::Attribute(Attribute::new("a", 1)),
    ///     Structure::Block(Block::new("b")),
    ///     Structure::Attribute(Attribute::new("c", 3)),
    /// ]);
    ///
    /// // Update all attribute keys
    /// for attr in body.attributes_mut() {
    ///     attr.key = Identifier::new(format!("attr_{}", attr.key)).unwrap();
    /// }
    ///
    /// assert_eq!(body.into_inner(), [
    ///     Structure::Attribute(Attribute::new("attr_a", 1)),
    ///     Structure::Block(Block::new("b")),
    ///     Structure::Attribute(Attribute::new("attr_c", 3)),
    /// ]);
    /// ```
    pub fn attributes_mut(&mut self) -> AttributesMut<'_> {
        AttributesMut::new(self)
    }

    /// Creates a consuming iterator visiting all attributes within the `Body`. The object cannot
    /// be used after calling this. The iterator element type is `Attribute`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl::{Attribute, Block, Body, Structure};
    ///
    /// let body = Body::from([
    ///     Structure::Attribute(Attribute::new("a", 1)),
    ///     Structure::Block(Block::new("b")),
    ///     Structure::Attribute(Attribute::new("c", 3)),
    /// ]);
    ///
    /// let vec: Vec<Attribute> = body.into_attributes().collect();
    /// assert_eq!(vec, [Attribute::new("a", 1), Attribute::new("c", 3)]);
    /// ```
    pub fn into_attributes(self) -> IntoAttributes {
        IntoAttributes::new(self)
    }

    /// An iterator visiting all blocks within the `Body`. The iterator element type is `&'a
    /// Block`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl::{Attribute, Block, Body, Structure};
    ///
    /// let body = Body::from([
    ///     Structure::Attribute(Attribute::new("a", 1)),
    ///     Structure::Block(Block::new("b")),
    ///     Structure::Attribute(Attribute::new("c", 3)),
    /// ]);
    ///
    /// let vec: Vec<&Block> = body.blocks().collect();
    /// assert_eq!(vec, [&Block::new("b")]);
    /// ```
    pub fn blocks(&self) -> Blocks<'_> {
        Blocks::new(self)
    }

    /// An iterator visiting all blocks within the `Body`. The iterator element type is `&'a mut
    /// Block`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl::{Attribute, Block, Body, Identifier, Structure};
    ///
    /// let mut body = Body::from([
    ///     Structure::Attribute(Attribute::new("a", 1)),
    ///     Structure::Block(Block::new("b")),
    ///     Structure::Attribute(Attribute::new("c", 3)),
    /// ]);
    ///
    /// // Update all block identifiers
    /// for block in body.blocks_mut() {
    ///     block.identifier = Identifier::new(format!("block_{}", block.identifier)).unwrap();
    /// }
    ///
    /// assert_eq!(body.into_inner(), [
    ///     Structure::Attribute(Attribute::new("a", 1)),
    ///     Structure::Block(Block::new("block_b")),
    ///     Structure::Attribute(Attribute::new("c", 3)),
    /// ]);
    /// ```
    pub fn blocks_mut(&mut self) -> BlocksMut<'_> {
        BlocksMut::new(self)
    }

    /// Creates a consuming iterator visiting all blocks within the `Body`. The object cannot
    /// be used after calling this. The iterator element type is `Block`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl::{Attribute, Block, Body, Structure};
    ///
    /// let body = Body::from([
    ///     Structure::Attribute(Attribute::new("a", 1)),
    ///     Structure::Block(Block::new("b")),
    ///     Structure::Attribute(Attribute::new("c", 3)),
    /// ]);
    ///
    /// let vec: Vec<Block> = body.into_blocks().collect();
    /// assert_eq!(vec, [Block::new("b")]);
    /// ```
    pub fn into_blocks(self) -> IntoBlocks {
        IntoBlocks::new(self)
    }
}

impl<T> From<T> for Body
where
    T: Into<Structure>,
{
    fn from(value: T) -> Body {
        Body(vec![value.into()])
    }
}

impl<T> From<Vec<T>> for Body
where
    T: Into<Structure>,
{
    fn from(vec: Vec<T>) -> Self {
        Body::from_iter(vec)
    }
}

impl<T> From<&[T]> for Body
where
    T: Clone + Into<Structure>,
{
    fn from(slice: &[T]) -> Self {
        Body::from_iter(slice.to_vec())
    }
}

impl<T> From<&mut [T]> for Body
where
    T: Clone + Into<Structure>,
{
    fn from(slice: &mut [T]) -> Self {
        Body::from_iter(slice.to_vec())
    }
}

impl<T, const N: usize> From<[T; N]> for Body
where
    T: Into<Structure>,
{
    fn from(arr: [T; N]) -> Self {
        Body::from_iter(arr)
    }
}

/// `BodyBuilder` builds a HCL [`Body`].
///
/// The builder allows to build the `Body` by adding attributes and other nested blocks via chained
/// method calls. A call to [`.build()`](BodyBuilder::build) produces the final `Body`.
///
/// ## Example
///
/// ```
/// use hcl::{Body, Block};
///
/// let body = Body::builder()
///     .add_block(
///         Block::builder("resource")
///             .add_label("aws_s3_bucket")
///             .add_label("mybucket")
///             .add_attribute(("name", "mybucket"))
///             .build()
///     )
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct BodyBuilder(Vec<Structure>);

impl BodyBuilder {
    /// Adds an `Attribute` to the body.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_attribute<A>(self, attr: A) -> BodyBuilder
    where
        A: Into<Attribute>,
    {
        self.add_structure(attr.into())
    }

    /// Adds `Attribute`s to the body from an iterator.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_attributes<I>(self, iter: I) -> BodyBuilder
    where
        I: IntoIterator,
        I::Item: Into<Attribute>,
    {
        self.add_structures(iter.into_iter().map(Into::into))
    }

    /// Adds a `Block` to the body.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_block<B>(self, block: B) -> BodyBuilder
    where
        B: Into<Block>,
    {
        self.add_structure(block.into())
    }

    /// Adds `Block`s to the body from an iterator.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_blocks<I>(self, iter: I) -> BodyBuilder
    where
        I: IntoIterator,
        I::Item: Into<Block>,
    {
        self.add_structures(iter.into_iter().map(Into::into))
    }

    /// Adds a `Structure` to the body.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_structure<S>(mut self, structure: S) -> BodyBuilder
    where
        S: Into<Structure>,
    {
        self.0.push(structure.into());
        self
    }

    /// Adds `Structure`s to the body from an iterator.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_structures<I>(mut self, iter: I) -> BodyBuilder
    where
        I: IntoIterator,
        I::Item: Into<Structure>,
    {
        self.0.extend(iter.into_iter().map(Into::into));
        self
    }

    /// Consumes `self` and builds the [`Body`] from the structures added via the builder methods.
    pub fn build(self) -> Body {
        Body::from_iter(self.0)
    }
}
