use crate::encode::{EncodeDecorated, EncodeState, NO_DECOR};
use crate::format::{Format, Formatter};
use crate::structure::{Attribute, AttributeMut, Block, Structure, StructureMut};
use crate::visit_mut::VisitMut;
use crate::{parser, Decor};
use std::fmt;
use std::ops::Range;
use std::str::FromStr;

/// An owning iterator over the elements of a `Body`.
///
/// Values of this type are created by the [`into_iter`] method on [`Body`] (provided by the
/// [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
/// [`IntoIterator`]: core::iter::IntoIterator
pub type IntoIter = Box<dyn Iterator<Item = Structure>>;

/// An iterator over the elements of a `Body`.
///
/// Values of this type are created by the [`iter`] method on [`Body`]. See its documentation
/// for more.
///
/// [`iter`]: Body::iter
pub type Iter<'a> = Box<dyn Iterator<Item = &'a Structure> + 'a>;

/// A mutable iterator over the elements of a `Body`.
///
/// Values of this type are created by the [`iter_mut`] method on [`Body`]. See its
/// documentation for more.
///
/// [`iter_mut`]: Body::iter_mut
pub type IterMut<'a> = Box<dyn Iterator<Item = StructureMut<'a>> + 'a>;

/// An owning iterator over the `Attribute`s within a `Body`.
///
/// Values of this type are created by the [`into_attributes`] method on [`Body`]. See its
/// documentation for more.
///
/// [`into_attributes`]: Body::into_attributes
pub type IntoAttributes = Box<dyn Iterator<Item = Attribute>>;

/// An iterator over the `Attribute`s within a `Body`.
///
/// Values of this type are created by the [`attributes`] method on [`Body`]. See its documentation
/// for more.
///
/// [`attributes`]: Body::attributes
pub type Attributes<'a> = Box<dyn Iterator<Item = &'a Attribute> + 'a>;

/// A mutable iterator over the `Attribute`s within a `Body`.
///
/// Values of this type are created by the [`attributes_mut`] method on [`Body`]. See its
/// documentation for more.
///
/// [`attributes_mut`]: Body::attributes_mut
pub type AttributesMut<'a> = Box<dyn Iterator<Item = AttributeMut<'a>> + 'a>;

/// An owning iterator over the `Block`s within a `Body`.
///
/// Values of this type are created by the [`into_blocks`] method on [`Body`]. See its
/// documentation for more.
///
/// [`into_blocks`]: Body::into_blocks
pub type IntoBlocks = Box<dyn Iterator<Item = Block>>;

/// An iterator over the `Block`s within a `Body`.
///
/// Values of this type are created by the [`blocks`] method on [`Body`]. See its documentation
/// for more.
///
/// [`blocks`]: Body::blocks
pub type Blocks<'a> = Box<dyn Iterator<Item = &'a Block> + 'a>;

/// A mutable iterator over the `Block`s within a `Body`.
///
/// Values of this type are created by the [`blocks_mut`] method on [`Body`]. See its
/// documentation for more.
///
/// [`blocks_mut`]: Body::blocks_mut
pub type BlocksMut<'a> = Box<dyn Iterator<Item = &'a mut Block> + 'a>;

/// Represents an HCL config file body.
///
/// A `Body` consists of zero or more [`Attribute`][crate::structure::Attribute] and
/// [`Block`][crate::structure::Block] HCL structures.
#[derive(Debug, Clone, Default, Eq)]
pub struct Body {
    structures: Vec<Structure>,
    prefer_oneline: bool,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl Body {
    /// Constructs a new, empty `Body`.
    #[inline]
    pub fn new() -> Self {
        Body::default()
    }

    /// Constructs a new, empty `Body` with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Body {
            structures: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }

    /// Creates a new [`BodyBuilder`] to start building a new `Body`.
    #[inline]
    pub fn builder() -> BodyBuilder {
        BodyBuilder::default()
    }

    /// Returns `true` if the body contains no structures.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.structures.is_empty()
    }

    /// Returns the number of structures in the body, also referred to as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        self.structures.len()
    }

    /// Clears the body, removing all structures.
    #[inline]
    pub fn clear(&mut self) {
        self.structures.clear();
    }

    /// Returns a reference to the structure at the given index, or `None` if the index is out of
    /// bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Structure> {
        self.structures.get(index)
    }

    /// Returns a mutable reference to the structure at the given index, or `None` if the index is
    /// out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Structure> {
        self.structures.get_mut(index)
    }

    /// Returns `true` if the body contains an attribute with given key.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::structure::{Attribute, Body};
    /// use hcl_edit::Ident;
    ///
    /// let body = Body::from_iter([Attribute::new(Ident::new("foo"), "bar")]);
    /// assert!(body.has_attribute("foo"));
    /// assert!(!body.has_attribute("bar"));
    /// ```
    #[inline]
    pub fn has_attribute(&self, key: &str) -> bool {
        self.get_attribute(key).is_some()
    }

    /// Returns `true` if the body contains blocks with given identifier.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::structure::{Block, Body};
    /// use hcl_edit::Ident;
    ///
    /// let body = Body::from_iter([Block::new(Ident::new("foo"))]);
    /// assert!(body.has_blocks("foo"));
    /// assert!(!body.has_blocks("bar"));
    /// ```
    #[inline]
    pub fn has_blocks(&self, ident: &str) -> bool {
        self.get_blocks(ident).next().is_some()
    }

    /// Returns a reference to the `Attribute` with given key if it exists, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::structure::{Attribute, Body};
    /// use hcl_edit::Ident;
    ///
    /// let mut body = Body::new();
    ///
    /// assert!(body.get_attribute("foo").is_none());
    ///
    /// let foo = Attribute::new(Ident::new("foo"), "bar");
    ///
    /// body.push(foo.clone());
    ///
    /// assert_eq!(body.get_attribute("foo"), Some(&foo));
    /// ```
    pub fn get_attribute(&self, key: &str) -> Option<&Attribute> {
        self.structures
            .iter()
            .filter_map(Structure::as_attribute)
            .find(|attr| attr.has_key(key))
    }

    /// Returns a mutable reference to the `Attribute` with given key if it exists, otherwise
    /// `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::expr::Expression;
    /// use hcl_edit::structure::{Attribute, Body};
    /// use hcl_edit::Ident;
    ///
    /// let mut body = Body::new();
    ///
    /// assert!(body.get_attribute("foo").is_none());
    ///
    /// let foo = Attribute::new(Ident::new("foo"), "bar");
    ///
    /// body.push(foo.clone());
    ///
    /// if let Some(mut attr) = body.get_attribute_mut("foo") {
    ///     *attr.value_mut() = Expression::from("baz");
    /// }
    ///
    /// assert_eq!(body.get_attribute("foo"), Some(&Attribute::new(Ident::new("foo"), "baz")));
    /// ```
    pub fn get_attribute_mut(&mut self, key: &str) -> Option<AttributeMut<'_>> {
        self.structures
            .iter_mut()
            .filter_map(Structure::as_attribute_mut)
            .find(|attr| attr.has_key(key))
            .map(AttributeMut::new)
    }

    /// Returns an iterator visiting all `Block`s with the given identifier. The iterator element
    /// type is `&'a Block`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hcl_edit::structure::Body;
    ///
    /// let input = r#"
    /// resource "aws_s3_bucket" "bucket" {}
    ///
    /// variable "name" {}
    ///
    /// resource "aws_instance" "instance" {}
    /// "#;
    ///
    /// let body: Body = input.parse()?;
    ///
    /// let resources: Body = body.get_blocks("resource").cloned().collect();
    ///
    /// let expected = r#"
    /// resource "aws_s3_bucket" "bucket" {}
    ///
    /// resource "aws_instance" "instance" {}
    /// "#;
    ///
    /// assert_eq!(resources.to_string(), expected);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_blocks<'a>(&'a self, ident: &'a str) -> Blocks<'a> {
        Box::new(
            self.structures
                .iter()
                .filter_map(Structure::as_block)
                .filter(|block| block.has_ident(ident)),
        )
    }

    /// Returns an iterator visiting all `Block`s with the given identifier. The iterator element
    /// type is `&'a mut Block`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hcl_edit::expr::{Traversal, TraversalOperator};
    /// use hcl_edit::structure::{Attribute, Body};
    /// use hcl_edit::Ident;
    ///
    /// let input = r#"
    /// resource "aws_s3_bucket" "bucket" {}
    ///
    /// variable "name" {}
    ///
    /// resource "aws_db_instance" "db_instance" {}
    /// "#;
    ///
    /// let mut body: Body = input.parse()?;
    ///
    /// for block in body.get_blocks_mut("resource") {
    ///     let operators = vec![TraversalOperator::GetAttr(Ident::new("name").into()).into()];
    ///     let value = Traversal::new(Ident::new("var"), operators);
    ///     block.body.push(Attribute::new(Ident::new("name"), value));
    /// }
    ///
    /// let expected = r#"
    /// resource "aws_s3_bucket" "bucket" { name = var.name }
    ///
    /// variable "name" {}
    ///
    /// resource "aws_db_instance" "db_instance" { name = var.name }
    /// "#;
    ///
    /// assert_eq!(body.to_string(), expected);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_blocks_mut<'a>(&'a mut self, ident: &'a str) -> BlocksMut<'a> {
        Box::new(
            self.structures
                .iter_mut()
                .filter_map(Structure::as_block_mut)
                .filter(|block| block.has_ident(ident)),
        )
    }

    /// Inserts a structure at position `index` within the body, shifting all structures after it
    /// to the right.
    ///
    /// If it is attempted to insert an `Attribute` which already exists in the body, it is ignored
    /// and not inserted. For a fallible variant of this function see [`Body::try_insert`].
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, structure: impl Into<Structure>) {
        _ = self.try_insert(index, structure);
    }

    /// Inserts a structure at position `index` within the body, shifting all structures after it
    /// to the right.
    ///
    /// # Errors
    ///
    /// If it is attempted to insert an `Attribute` which already exists in the body, it is not
    /// inserted and returned as the `Result`'s `Err` variant instead.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::structure::{Attribute, Body};
    /// use hcl_edit::Ident;
    ///
    /// let mut body = Body::new();
    ///
    /// body.push(Attribute::new(Ident::new("foo"), "bar"));
    /// assert!(body.try_insert(0, Attribute::new(Ident::new("bar"), "baz")).is_ok());
    /// assert_eq!(body.len(), 2);
    ///
    /// let duplicate_attr = Attribute::new(Ident::new("foo"), "baz");
    ///
    /// assert_eq!(body.try_insert(0, duplicate_attr.clone()), Err(duplicate_attr));
    /// ```
    #[inline]
    pub fn try_insert(
        &mut self,
        index: usize,
        structure: impl Into<Structure>,
    ) -> Result<(), Attribute> {
        match structure.into() {
            Structure::Attribute(attr) if self.has_attribute(&attr.key) => Err(attr),
            structure => {
                self.structures.insert(index, structure);
                Ok(())
            }
        }
    }

    /// Appends a structure to the back of the body.
    ///
    /// If it is attempted to append an `Attribute` which already exists in the body, it is ignored
    /// and not appended. For a fallible variant of this function see [`Body::try_push`].
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    #[inline]
    pub fn push(&mut self, structure: impl Into<Structure>) {
        _ = self.try_push(structure);
    }

    /// Appends a structure to the back of the body.
    ///
    /// # Errors
    ///
    /// If it is attempted to append an `Attribute` which already exists in the body, it is not
    /// appended and returned as the `Result`'s `Err` variant instead.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::structure::{Attribute, Body};
    /// use hcl_edit::Ident;
    ///
    /// let mut body = Body::new();
    ///
    /// assert!(body.try_push(Attribute::new(Ident::new("foo"), "bar")).is_ok());
    /// assert!(body.try_push(Attribute::new(Ident::new("bar"), "baz")).is_ok());
    /// assert_eq!(body.len(), 2);
    ///
    /// let duplicate_attr = Attribute::new(Ident::new("foo"), "baz");
    ///
    /// assert_eq!(body.try_push(duplicate_attr.clone()), Err(duplicate_attr));
    /// ```
    #[inline]
    pub fn try_push(&mut self, structure: impl Into<Structure>) -> Result<(), Attribute> {
        match structure.into() {
            Structure::Attribute(attr) if self.has_attribute(&attr.key) => Err(attr),
            structure => {
                self.structures.push(structure);
                Ok(())
            }
        }
    }

    /// Removes the last structure from the body and returns it, or [`None`] if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<Structure> {
        self.structures.pop()
    }

    /// Removes and returns the structure at position `index` within the body, shifting all
    /// elements after it to the left.
    ///
    /// Like `Vec::remove`, the structure is removed by shifting all of the structures that follow
    /// it, preserving their relative order. **This perturbs the index of all of those elements!**
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Structure {
        self.structures.remove(index)
    }

    /// Removes and returns the attribute with given `key`.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::structure::{Attribute, Block, Body};
    /// use hcl_edit::Ident;
    ///
    /// let mut body = Body::new();
    /// body.push(Block::new(Ident::new("block")));
    ///
    /// assert!(body.remove_attribute("foo").is_none());
    ///
    /// let foo = Attribute::new(Ident::new("foo"), "bar");
    ///
    /// body.push(foo.clone());
    ///
    /// assert_eq!(body.len(), 2);
    /// assert_eq!(body.remove_attribute("foo"), Some(foo));
    /// assert_eq!(body.len(), 1);
    /// ```
    pub fn remove_attribute(&mut self, key: &str) -> Option<Attribute> {
        self.structures
            .iter()
            .position(|structure| {
                structure
                    .as_attribute()
                    .map_or(false, |attr| attr.has_key(key))
            })
            .and_then(|index| self.remove(index).into_attribute().ok())
    }

    /// Removes and returns all blocks with given `ident`.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::structure::{Attribute, Block, Body};
    /// use hcl_edit::Ident;
    ///
    /// let mut body = Body::builder()
    ///     .attribute(Attribute::new(Ident::new("foo"), "bar"))
    ///     .block(
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket"])
    ///     )
    ///     .block(Block::builder(Ident::new("variable")).label("name"))
    ///     .block(
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_db_instance", "db_instance"])
    ///     )
    ///     .build();
    ///
    /// let resources = body.remove_blocks("resource");
    ///
    /// assert_eq!(
    ///     resources,
    ///     vec![
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket"])
    ///             .build(),
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_db_instance", "db_instance"])
    ///             .build()
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     body,
    ///     Body::builder()
    ///         .attribute(Attribute::new(Ident::new("foo"), "bar"))
    ///         .block(Block::builder(Ident::new("variable")).label("name"))
    ///         .build()
    /// );
    /// ```
    pub fn remove_blocks(&mut self, ident: &str) -> Vec<Block> {
        let mut removed = Vec::new();

        while let Some(block) = self.remove_block(ident) {
            removed.push(block);
        }

        removed
    }

    fn remove_block(&mut self, ident: &str) -> Option<Block> {
        self.structures
            .iter()
            .position(|structure| {
                structure
                    .as_block()
                    .map_or(false, |block| block.has_ident(ident))
            })
            .and_then(|index| self.remove(index).into_block().ok())
    }

    /// An iterator visiting all body structures in insertion order. The iterator element type is
    /// `&'a Structure`.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.structures.iter())
    }

    /// An iterator visiting all body structures in insertion order, with mutable references to the
    /// values. The iterator element type is `StructureMut<'a>`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.structures.iter_mut().map(StructureMut::new))
    }

    /// An owning iterator visiting all `Attribute`s within the body in insertion order. The
    /// iterator element type is `Attribute`.
    #[inline]
    pub fn into_attributes(self) -> IntoAttributes {
        Box::new(
            self.structures
                .into_iter()
                .filter_map(|s| s.into_attribute().ok()),
        )
    }

    /// An iterator visiting all `Attribute`s within the body in insertion order. The iterator
    /// element type is `&'a Attribute`.
    #[inline]
    pub fn attributes(&self) -> Attributes<'_> {
        Box::new(self.structures.iter().filter_map(Structure::as_attribute))
    }

    /// An iterator visiting all `Attribute`s within the body in insertion order, with mutable
    /// references to the values. The iterator element type is `AttributeMut<'a>`.
    #[inline]
    pub fn attributes_mut(&mut self) -> AttributesMut<'_> {
        Box::new(
            self.structures
                .iter_mut()
                .filter_map(Structure::as_attribute_mut)
                .map(AttributeMut::new),
        )
    }

    /// An owning iterator visiting all `Block`s within the body in insertion order. The iterator
    /// element type is `Block`.
    #[inline]
    pub fn into_blocks(self) -> IntoBlocks {
        Box::new(
            self.structures
                .into_iter()
                .filter_map(|s| s.into_block().ok()),
        )
    }

    /// An iterator visiting all `Block`s within the body in insertion order. The iterator element
    /// type is `&'a Block`.
    #[inline]
    pub fn blocks(&self) -> Blocks<'_> {
        Box::new(self.structures.iter().filter_map(Structure::as_block))
    }

    /// An iterator visiting all `Block`s within the body in insertion order, with mutable
    /// references to the values. The iterator element type is `&'a mut Block`.
    #[inline]
    pub fn blocks_mut(&mut self) -> BlocksMut<'_> {
        Box::new(
            self.structures
                .iter_mut()
                .filter_map(Structure::as_block_mut),
        )
    }

    /// Configures whether the body should be displayed on a single line.
    ///
    /// This is only a hint which will be applied if the `Body` is part of a `Block` (that is: not
    /// the document root) and only if either of these conditions meet:
    ///
    /// - The body is empty. In this case, the opening (`{`) and closing (`}`) braces will be
    ///   places on the same line.
    /// - The body only consist of a single `Attribute`, which will be placed on the same
    ///   line as the opening and closing braces.
    ///
    /// In all other cases this hint is ignored.
    #[inline]
    pub fn set_prefer_oneline(&mut self, yes: bool) {
        self.prefer_oneline = yes;
    }

    /// Returns `true` if the body should be displayed on a single line.
    ///
    /// See the documentation of [`Body::set_prefer_oneline`] for more.
    #[inline]
    pub fn prefer_oneline(&self) -> bool {
        self.prefer_oneline
    }

    /// Returns `true` if the body only consist of a single `Attribute`.
    #[inline]
    pub(crate) fn has_single_attribute(&self) -> bool {
        self.len() == 1 && self.get(0).map_or(false, Structure::is_attribute)
    }

    pub(crate) fn from_vec_unchecked(structures: Vec<Structure>) -> Self {
        Body {
            structures,
            ..Default::default()
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        for structure in &mut self.structures {
            structure.despan(input);
        }
    }
}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.structures == other.structures
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode_decorated(&mut state, NO_DECOR)
    }
}

impl FromStr for Body {
    type Err = parser::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse_body(s)
    }
}

impl From<Vec<Structure>> for Body {
    fn from(structures: Vec<Structure>) -> Self {
        Body::from_iter(structures)
    }
}

impl<T> Extend<T> for Body
where
    T: Into<Structure>,
{
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iterable.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.structures.reserve(reserve);
        iter.for_each(|v| self.push(v));
    }
}

impl<T> FromIterator<T> for Body
where
    T: Into<Structure>,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let mut body = Body::with_capacity(lower);
        body.extend(iter);
        body
    }
}

impl IntoIterator for Body {
    type Item = Structure;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.structures.into_iter())
    }
}

impl<'a> IntoIterator for &'a Body {
    type Item = &'a Structure;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Body {
    type Item = StructureMut<'a>;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl Format for Body {
    fn format(&mut self, mut fmt: Formatter) {
        fmt.visit_body_mut(self);
    }
}

decorate_impl!(Body);
span_impl!(Body);

/// `BodyBuilder` builds a HCL [`Body`].
///
/// The builder allows to build the `Body` by adding attributes and other nested blocks via chained
/// method calls. A call to [`.build()`](BodyBuilder::build) produces the final `Body`.
///
/// ## Example
///
/// ```
/// use hcl_edit::structure::{Attribute, Block, Body};
/// use hcl_edit::Ident;
///
/// let body = Body::builder()
///     .block(
///         Block::builder(Ident::new("resource"))
///             .labels(["aws_s3_bucket", "mybucket"])
///             .attribute(Attribute::new(Ident::new("name"), "mybucket"))
///     )
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct BodyBuilder {
    body: Body,
}

impl BodyBuilder {
    /// Adds an `Attribute` to the body.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    #[inline]
    pub fn attribute(self, attr: impl Into<Attribute>) -> BodyBuilder {
        self.structure(attr.into())
    }

    /// Adds `Attribute`s to the body from an iterator.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    #[inline]
    pub fn attributes<I>(self, iter: I) -> BodyBuilder
    where
        I: IntoIterator,
        I::Item: Into<Attribute>,
    {
        self.structures(iter.into_iter().map(Into::into))
    }

    /// Adds a `Block` to the body.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    #[inline]
    pub fn block(self, block: impl Into<Block>) -> BodyBuilder {
        self.structure(block.into())
    }

    /// Adds `Block`s to the body from an iterator.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    #[inline]
    pub fn blocks<I>(self, iter: I) -> BodyBuilder
    where
        I: IntoIterator,
        I::Item: Into<Block>,
    {
        self.structures(iter.into_iter().map(Into::into))
    }

    /// Adds a `Structure` to the body.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    #[inline]
    pub fn structure(mut self, structure: impl Into<Structure>) -> BodyBuilder {
        self.body.push(structure.into());
        self
    }

    /// Adds `Structure`s to the body from an iterator.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    #[inline]
    pub fn structures<I>(mut self, iter: I) -> BodyBuilder
    where
        I: IntoIterator,
        I::Item: Into<Structure>,
    {
        self.body.extend(iter);
        self
    }

    /// Consumes `self` and builds the [`Body`] from the structures added via the builder methods.
    #[inline]
    pub fn build(self) -> Body {
        self.body
    }
}

impl From<BodyBuilder> for Body {
    #[inline]
    fn from(builder: BodyBuilder) -> Self {
        builder.build()
    }
}
