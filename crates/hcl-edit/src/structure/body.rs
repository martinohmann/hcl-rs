use crate::encode::{EncodeDecorated, EncodeState, NO_DECOR};
use crate::parser;
use crate::repr::{Decor, Decorate, SetSpan, Span};
use crate::structure::{Attribute, Block, BlockLabelSelector, Structure};
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
pub type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Structure> + 'a>;

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
pub type AttributesMut<'a> = Box<dyn Iterator<Item = &'a mut Attribute> + 'a>;

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
            .find(|attr| attr.key.as_str() == key)
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
    /// if let Some(attr) = body.get_attribute_mut("foo") {
    ///     attr.value = Expression::from("baz");
    /// }
    ///
    /// assert_eq!(body.get_attribute("foo"), Some(&Attribute::new(Ident::new("foo"), "baz")));
    /// ```
    pub fn get_attribute_mut(&mut self, key: &str) -> Option<&mut Attribute> {
        self.structures
            .iter_mut()
            .filter_map(Structure::as_attribute_mut)
            .find(|attr| attr.key.as_str() == key)
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
                .filter(move |block| block.ident.as_str() == ident),
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
                .filter(move |block| block.ident.as_str() == ident),
        )
    }

    /// Returns an iterator visiting all `Block`s with the given identifier matching the provided
    /// label selector. The iterator element type is `&'a Block`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hcl_edit::structure::Body;
    ///
    /// let input = r#"
    /// resource "aws_s3_bucket" "bucket1" {}
    /// resource "aws_db_instance" "db" {}
    ///
    /// variable "name" {}
    ///
    /// resource "aws_s3_bucket" "bucket2" {}
    /// "#;
    ///
    /// let body: Body = input.parse()?;
    ///
    /// let resources: Body = body.get_labeled_blocks("resource", "aws_s3_bucket").cloned().collect();
    ///
    /// let expected = r#"
    /// resource "aws_s3_bucket" "bucket1" {}
    ///
    /// resource "aws_s3_bucket" "bucket2" {}
    /// "#;
    ///
    /// assert_eq!(resources.to_string(), expected);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_labeled_blocks<'a, S>(&'a self, ident: &'a str, selector: S) -> Blocks<'a>
    where
        S: BlockLabelSelector + Copy + 'a,
    {
        Box::new(
            self.structures
                .iter()
                .filter_map(Structure::as_block)
                .filter(move |block| {
                    block.ident.as_str() == ident && selector.matches_labels(&block.labels)
                }),
        )
    }

    /// Returns an iterator visiting all `Block`s with the given identifier matching the provided
    /// label selector. The iterator element type is `&'a mut Block`.
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
    /// resource "aws_s3_bucket" "bucket1" {}
    /// resource "aws_db_instance" "db_instance" {}
    ///
    /// variable "name" {}
    ///
    /// resource "aws_s3_bucket" "bucket2" {}
    /// "#;
    ///
    /// let mut body: Body = input.parse()?;
    ///
    /// for block in body.get_labeled_blocks_mut("resource", "aws_s3_bucket") {
    ///     let operators = vec![TraversalOperator::GetAttr(Ident::new("name").into()).into()];
    ///     let value = Traversal::new(Ident::new("var"), operators);
    ///     block.body.push(Attribute::new(Ident::new("name"), value));
    /// }
    ///
    /// let expected = r#"
    /// resource "aws_s3_bucket" "bucket1" { name = var.name }
    /// resource "aws_db_instance" "db_instance" {}
    ///
    /// variable "name" {}
    ///
    /// resource "aws_s3_bucket" "bucket2" { name = var.name }
    /// "#;
    ///
    /// assert_eq!(body.to_string(), expected);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn get_labeled_blocks_mut<'a, S>(&'a mut self, ident: &'a str, selector: S) -> BlocksMut<'a>
    where
        S: BlockLabelSelector + Copy + 'a,
    {
        Box::new(
            self.structures
                .iter_mut()
                .filter_map(Structure::as_block_mut)
                .filter(move |block| {
                    block.ident.as_str() == ident && selector.matches_labels(&block.labels)
                }),
        )
    }

    /// Inserts a structure at position `index` within the body, shifting all structures after it
    /// to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, structure: impl Into<Structure>) {
        self.structures.insert(index, structure.into());
    }

    /// Appends a structure to the back of the body.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    #[inline]
    pub fn push(&mut self, structure: impl Into<Structure>) {
        self.structures.push(structure.into());
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
        self.remove_first(|structure| {
            structure
                .as_attribute()
                .map_or(false, |attr| attr.key.as_str() == key)
        })
        .and_then(Structure::into_attribute)
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

    /// Removes and returns all blocks with given `ident` matching the provided label selector.
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
    ///             .labels(["aws_s3_bucket", "bucket1"])
    ///     )
    ///     .block(Block::builder(Ident::new("variable")).label("name"))
    ///     .block(
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_db_instance", "db_instance"])
    ///     )
    ///     .block(
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket2"])
    ///     )
    ///     .build();
    ///
    /// let resources = body.remove_labeled_blocks("resource", "aws_s3_bucket");
    ///
    /// assert_eq!(
    ///     resources,
    ///     vec![
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket1"])
    ///             .build(),
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket2"])
    ///             .build(),
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     body,
    ///     Body::builder()
    ///         .attribute(Attribute::new(Ident::new("foo"), "bar"))
    ///         .block(Block::builder(Ident::new("variable")).label("name"))
    ///         .block(
    ///             Block::builder(Ident::new("resource"))
    ///                 .labels(["aws_db_instance", "db_instance"])
    ///         )
    ///         .build()
    /// );
    /// ```
    pub fn remove_labeled_blocks<S>(&mut self, ident: &str, selector: S) -> Vec<Block>
    where
        S: BlockLabelSelector + Copy,
    {
        let mut removed = Vec::new();

        while let Some(block) = self.remove_labeled_block(ident, selector) {
            removed.push(block);
        }

        removed
    }

    fn remove_first<P>(&mut self, predicate: P) -> Option<Structure>
    where
        P: FnMut(&Structure) -> bool,
    {
        self.structures
            .iter()
            .position(predicate)
            .map(|index| self.remove(index))
    }

    fn remove_block(&mut self, ident: &str) -> Option<Block> {
        self.remove_first(|structure| {
            structure
                .as_block()
                .map_or(false, |block| block.ident.as_str() == ident)
        })
        .and_then(Structure::into_block)
    }

    fn remove_labeled_block<S>(&mut self, ident: &str, selector: S) -> Option<Block>
    where
        S: BlockLabelSelector + Copy,
    {
        self.remove_first(|structure| {
            structure.as_block().map_or(false, |block| {
                block.ident.as_str() == ident && selector.matches_labels(&block.labels)
            })
        })
        .and_then(Structure::into_block)
    }

    /// An iterator visiting all body structures in insertion order. The iterator element type is
    /// `&'a Structure`.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.structures.iter())
    }

    /// An iterator visiting all body structures in insertion order, with mutable references to the
    /// values. The iterator element type is `&'a mut Structure`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.structures.iter_mut())
    }

    /// An owning iterator visiting all `Attribute`s within the body in insertion order. The
    /// iterator element type is `Attribute`.
    #[inline]
    pub fn into_attributes(self) -> IntoAttributes {
        Box::new(
            self.structures
                .into_iter()
                .filter_map(Structure::into_attribute),
        )
    }

    /// An iterator visiting all `Attribute`s within the body in insertion order. The iterator
    /// element type is `&'a Attribute`.
    #[inline]
    pub fn attributes(&self) -> Attributes<'_> {
        Box::new(self.structures.iter().filter_map(Structure::as_attribute))
    }

    /// An iterator visiting all `Attribute`s within the body in insertion order, with mutable
    /// references to the values. The iterator element type is `&'a mut Attribute`.
    #[inline]
    pub fn attributes_mut(&mut self) -> AttributesMut<'_> {
        Box::new(
            self.structures
                .iter_mut()
                .filter_map(Structure::as_attribute_mut),
        )
    }

    /// An owning iterator visiting all `Block`s within the body in insertion order. The iterator
    /// element type is `Block`.
    #[inline]
    pub fn into_blocks(self) -> IntoBlocks {
        Box::new(
            self.structures
                .into_iter()
                .filter_map(Structure::into_block),
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
        Body {
            structures,
            ..Default::default()
        }
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
    type Item = &'a mut Structure;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
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
    pub fn structure(mut self, structures: impl Into<Structure>) -> BodyBuilder {
        self.body.push(structures.into());
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
