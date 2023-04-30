//! Types to represent the HCL structural sub-language.

use crate::encode::{EncodeDecorated, EncodeState, NO_DECOR};
use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::{parser, Ident, RawString};
use std::fmt;
use std::ops::{self, Range};
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

/// Represents an HCL config file body.
///
/// A `Body` consists of zero or more [`Attribute`] and [`Block`] HCL structures.
#[derive(Debug, Clone, Default, Eq)]
pub struct Body {
    structures: Vec<Structure>,
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

/// Represents an HCL attribute which consists of an attribute key and a value expression.
///
/// In HCL syntax this is represented as:
///
/// ```hcl
/// key = value
/// ```
///
/// Use [`Attribute::new`] to construct an [`Attribute`] from a value that is convertible to this
/// crate's [`Expression`] type.
#[derive(Debug, Clone, Eq)]
pub struct Attribute {
    /// The HCL attribute's key.
    pub key: Decorated<Ident>,
    /// The value expression of the HCL attribute.
    pub value: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl Attribute {
    /// Creates a new `Attribute` from a key and a value.
    pub fn new(key: Decorated<Ident>, value: Expression) -> Attribute {
        Attribute {
            key,
            value,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.key.decor_mut().despan(input);
        self.value.despan(input);
    }
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

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
#[derive(Debug, Clone, Eq)]
pub struct Block {
    /// The block identifier.
    pub ident: Decorated<Ident>,
    /// Zero or more block labels.
    pub labels: Vec<BlockLabel>,
    /// Represents the `Block`'s body.
    pub body: BlockBody,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl Block {
    /// Creates a new `Block` from an identifier and a block body.
    pub fn new(ident: Decorated<Ident>, body: BlockBody) -> Block {
        Block {
            ident,
            labels: Vec::new(),
            body,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.ident.decor_mut().despan(input);
        for label in &mut self.labels {
            label.despan(input);
        }
        self.body.despan(input);
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident && self.labels == other.labels && self.body == other.body
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockLabel {
    /// A bare HCL block label.
    Ident(Decorated<Ident>),
    /// A quoted string literal.
    String(Decorated<String>),
}

impl BlockLabel {
    /// Returns `true` if the block label is an identifier.
    pub fn is_ident(&self) -> bool {
        matches!(self, BlockLabel::Ident(_))
    }

    /// Returns `true` if the block label is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, BlockLabel::String(_))
    }

    /// Returns a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        match self {
            BlockLabel::Ident(ident) => ident.as_str(),
            BlockLabel::String(string) => string.as_str(),
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            BlockLabel::Ident(ident) => ident.decor_mut().despan(input),
            BlockLabel::String(string) => string.decor_mut().despan(input),
        }
    }
}

impl From<Ident> for BlockLabel {
    fn from(value: Ident) -> Self {
        BlockLabel::from(Decorated::new(value))
    }
}

impl From<Decorated<Ident>> for BlockLabel {
    fn from(value: Decorated<Ident>) -> Self {
        BlockLabel::Ident(value)
    }
}

impl From<&str> for BlockLabel {
    fn from(value: &str) -> Self {
        BlockLabel::from(value.to_string())
    }
}

impl From<String> for BlockLabel {
    fn from(value: String) -> Self {
        BlockLabel::from(Decorated::new(value))
    }
}

impl From<Decorated<String>> for BlockLabel {
    fn from(value: Decorated<String>) -> Self {
        BlockLabel::String(value)
    }
}

impl AsRef<str> for BlockLabel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ops::Deref for BlockLabel {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

/// Represents an HCL block body.
///
/// This can be either a multiline body with zero or more [`Structure`]s, or a oneline body
/// containing zero or one [`Attribute`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockBody {
    /// A multiline block body with zero or more [`Structure`]s.
    Multiline(Body),
    /// A oneline block body with zero or one [`Attribute`]s.
    Oneline(Box<OnelineBody>),
}

impl BlockBody {
    /// Returns `true` if the block body contains no structures.
    pub fn is_empty(&self) -> bool {
        match self {
            BlockBody::Multiline(body) => body.is_empty(),
            BlockBody::Oneline(oneline) => oneline.is_empty(),
        }
    }

    /// Returns the number of structures in the block body, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        match self {
            BlockBody::Multiline(body) => body.len(),
            BlockBody::Oneline(oneline) => {
                if oneline.is_empty() {
                    0
                } else {
                    1
                }
            }
        }
    }

    /// Returns `true` if this is a multiline block body.
    pub fn is_multiline(&self) -> bool {
        self.as_multiline().is_some()
    }

    /// If the `BlockBody` is of variant `Multiline`, returns a reference to the [`Body`],
    /// otherwise `None`.
    pub fn as_multiline(&self) -> Option<&Body> {
        match self {
            BlockBody::Multiline(body) => Some(body),
            BlockBody::Oneline(_) => None,
        }
    }

    /// If the `BlockBody` is of variant `Multiline`, returns a mutable reference to the [`Body`],
    /// otherwise `None`.
    pub fn as_multiline_mut(&mut self) -> Option<&mut Body> {
        match self {
            BlockBody::Multiline(body) => Some(body),
            BlockBody::Oneline(_) => None,
        }
    }

    /// In-place converts into a multiline block body (if needed) and returns a mutable reference
    /// to it.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hcl_edit::structure::{Attribute, BlockBody, OnelineBody, Structure};
    /// use hcl_edit::Ident;
    ///
    /// let attr = Attribute::new(Ident::new("key")?.into(), "value".into());
    /// let oneline = OnelineBody::from(attr.clone());
    /// let mut block_body = BlockBody::from(oneline);
    ///
    /// assert!(block_body.is_oneline());
    ///
    /// let multiline = block_body.make_multiline();
    ///
    /// assert_eq!(multiline.len(), 1);
    /// assert_eq!(multiline.get(0), Some(&Structure::Attribute(attr)));
    ///
    /// assert!(block_body.is_multiline());
    /// #   Ok(())
    /// # }
    /// ```
    pub fn make_multiline(&mut self) -> &mut Body {
        if let BlockBody::Oneline(oneline) = self {
            let mut body = Body::with_capacity(oneline.len());
            body.extend(oneline.iter().cloned());
            *self = BlockBody::Multiline(body);
        }

        match self {
            BlockBody::Multiline(body) => body,
            BlockBody::Oneline(_) => unreachable!(),
        }
    }

    /// Converts into a multiline block body (if needed) and returns the result.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hcl_edit::structure::{Attribute, BlockBody, OnelineBody, Structure};
    /// use hcl_edit::Ident;
    ///
    /// let attr = Attribute::new(Ident::new("key")?.into(), "value".into());
    /// let oneline = OnelineBody::from(attr.clone());
    /// let block_body = BlockBody::from(oneline);
    ///
    /// assert!(block_body.is_oneline());
    ///
    /// let multiline = block_body.into_multiline();
    ///
    /// assert_eq!(multiline.len(), 1);
    /// assert_eq!(multiline.get(0), Some(&Structure::Attribute(attr)));
    /// #   Ok(())
    /// # }
    /// ```
    pub fn into_multiline(self) -> Body {
        match self {
            BlockBody::Multiline(body) => body,
            BlockBody::Oneline(oneline) => {
                let mut body = Body::with_capacity(oneline.len());
                body.extend(oneline.into_iter());
                body
            }
        }
    }

    /// Returns `true` if this is a oneline block body.
    pub fn is_oneline(&self) -> bool {
        self.as_oneline().is_some()
    }

    /// If the `BlockBody` is of variant `Oneline`, returns a reference to the [`OnelineBody`],
    /// otherwise `None`.
    pub fn as_oneline(&self) -> Option<&OnelineBody> {
        match self {
            BlockBody::Multiline(_) => None,
            BlockBody::Oneline(oneline) => Some(oneline),
        }
    }

    /// If the `BlockBody` is of variant `Oneline`, returns a mutable reference to the
    /// [`OnelineBody`], otherwise `None`.
    pub fn as_oneline_mut(&mut self) -> Option<&mut OnelineBody> {
        match self {
            BlockBody::Multiline(_) => None,
            BlockBody::Oneline(oneline) => Some(oneline),
        }
    }

    /// In-place converts into a oneline block body (if needed) and returns a mutable reference
    /// to it.
    ///
    /// # Errors
    ///
    /// The conversion may fail under the following conditions:
    ///
    /// - The block body contains more than 1 `Structure`.
    /// - The block body contains exactly one `Structure`, but it is a `Block`.
    ///
    /// In both cases a mutable reference to the original `BlockBody` is returned as error.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hcl_edit::structure::{Attribute, BlockBody, Body, Structure};
    /// use hcl_edit::Ident;
    ///
    /// let attr = Attribute::new(Ident::new("key")?.into(), "value".into());
    /// let mut multiline = Body::new();
    /// multiline.push(attr.clone());
    ///
    /// let mut block_body = BlockBody::from(multiline);
    ///
    /// assert!(block_body.is_multiline());
    ///
    /// let oneline = block_body.make_oneline().unwrap();
    ///
    /// assert_eq!(oneline.len(), 1);
    /// assert_eq!(oneline.as_attribute(), Some(&attr));
    ///
    /// assert!(block_body.is_oneline());
    /// #   Ok(())
    /// # }
    /// ```
    pub fn make_oneline(&mut self) -> Result<&mut OnelineBody, &mut Self> {
        if let BlockBody::Multiline(body) = self {
            if body.len() > 1 {
                return Err(self);
            }

            let oneline = match body.get(0) {
                None => OnelineBody::new(),
                Some(Structure::Attribute(attr)) => OnelineBody::from(attr.clone()),
                Some(Structure::Block(_)) => return Err(self),
            };

            *self = BlockBody::from(oneline);
        }

        match self {
            BlockBody::Multiline(_) => Err(self),
            BlockBody::Oneline(oneline) => Ok(oneline),
        }
    }

    /// Converts into a oneline block body (if needed) and returns the result.
    ///
    /// # Errors
    ///
    /// The conversion may fail under the following conditions:
    ///
    /// - The block body contains more than 1 `Structure`.
    /// - The block body contains exactly one `Structure`, but it is a `Block`.
    ///
    /// In both cases the original `BlockBody` is returned as error.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hcl_edit::structure::{Attribute, BlockBody, Body, Structure};
    /// use hcl_edit::Ident;
    ///
    /// let attr = Attribute::new(Ident::new("key")?.into(), "value".into());
    /// let mut multiline = Body::new();
    /// multiline.push(attr.clone());
    ///
    /// let block_body = BlockBody::from(multiline);
    ///
    /// assert!(block_body.is_multiline());
    ///
    /// let oneline = block_body.into_oneline().unwrap();
    ///
    /// assert_eq!(oneline.len(), 1);
    /// assert_eq!(oneline.as_attribute(), Some(&attr));
    /// #   Ok(())
    /// # }
    /// ```
    pub fn into_oneline(self) -> Result<OnelineBody, Self> {
        match self {
            BlockBody::Oneline(oneline) => Ok(*oneline),
            BlockBody::Multiline(body) if body.len() <= 1 => match body.get(0) {
                Some(Structure::Attribute(attr)) => Ok(OnelineBody::from(attr.clone())),
                Some(Structure::Block(_)) => Err(BlockBody::Multiline(body)),
                None => Ok(OnelineBody::new()),
            },
            BlockBody::Multiline(body) => Err(BlockBody::Multiline(body)),
        }
    }

    /// An iterator visiting all body structures in insertion order. The iterator element type is
    /// `&'a Structure`.
    pub fn iter(&self) -> Iter<'_> {
        match self {
            BlockBody::Multiline(body) => body.iter(),
            BlockBody::Oneline(oneline) => oneline.iter(),
        }
    }

    /// An iterator visiting all body structures in insertion order, with mutable references to the
    /// values. The iterator element type is `&'a mut Structure`.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        match self {
            BlockBody::Multiline(body) => body.iter_mut(),
            BlockBody::Oneline(oneline) => oneline.iter_mut(),
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            BlockBody::Multiline(body) => body.despan(input),
            BlockBody::Oneline(oneline) => oneline.despan(input),
        }
    }
}

impl Default for BlockBody {
    fn default() -> Self {
        BlockBody::Multiline(Body::default())
    }
}

impl From<Body> for BlockBody {
    fn from(value: Body) -> Self {
        BlockBody::Multiline(value)
    }
}

impl From<OnelineBody> for BlockBody {
    fn from(value: OnelineBody) -> Self {
        BlockBody::Oneline(Box::new(value))
    }
}

impl<T> FromIterator<T> for BlockBody
where
    T: Into<Structure>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        BlockBody::Multiline(Body::from_iter(iter))
    }
}

impl IntoIterator for BlockBody {
    type Item = Structure;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            BlockBody::Multiline(body) => body.into_iter(),
            BlockBody::Oneline(oneline) => oneline.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a BlockBody {
    type Item = &'a Structure;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut BlockBody {
    type Item = &'a mut Structure;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Represents a oneline HCL block body containing zero or one [`Attribute`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OnelineBody {
    // Always of variant `Structure::Attribute` if not `None`. It's wrapped in a `Structure` to
    // support the creation of iterators over (mutable) `Structure` references in `BlockBody`.
    attr: Option<Structure>,
    trailing: RawString,
}

impl OnelineBody {
    /// Creates a new empty `OnelineBody`.
    pub fn new() -> OnelineBody {
        OnelineBody::default()
    }

    /// Returns `true` if the block body is empty.
    pub fn is_empty(&self) -> bool {
        self.attr.is_none()
    }

    /// Returns the number of structures in the block body, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            1
        }
    }

    /// Sets the optional [`Attribute`] within the online block body.
    pub fn set_attribute(&mut self, attr: impl Into<Attribute>) {
        self.attr = Some(Structure::Attribute(attr.into()));
    }

    /// If the `OnelineBody` contains an `Attribute`, returns a reference to it, otherwise `None`.
    pub fn as_attribute(&self) -> Option<&Attribute> {
        self.attr.as_ref().and_then(Structure::as_attribute)
    }

    /// If the `OnelineBody` contains an `Attribute`, returns a mutable reference to it, otherwise
    /// `None`.
    pub fn as_attribute_mut(&mut self) -> Option<&mut Attribute> {
        self.attr.as_mut().and_then(Structure::as_attribute_mut)
    }

    /// Return a reference to raw trailing decor before the block's closing `}`.
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    /// Set the raw trailing decor before the block's closing `}`.
    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    /// An iterator visiting all body structures in insertion order. The iterator element type is
    /// `&'a Structure` and is guaranteed to yield either zero or one elements of variant
    /// `Structure::Attribute`.
    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.attr.iter())
    }

    /// An iterator visiting all body structures in insertion order, with mutable references to the
    /// values. The iterator element type is `&'a mut Structure` and is guaranteed to yield either
    /// zero or one elements of variant `Structure::Attribute`.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.attr.iter_mut())
    }

    pub(crate) fn despan(&mut self, input: &str) {
        if let Some(attr) = &mut self.attr {
            attr.despan(input);
        }
        self.trailing.despan(input);
    }
}

impl From<Attribute> for OnelineBody {
    fn from(attr: Attribute) -> Self {
        OnelineBody {
            attr: Some(Structure::Attribute(attr)),
            trailing: RawString::default(),
        }
    }
}

impl IntoIterator for OnelineBody {
    type Item = Structure;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.attr.into_iter())
    }
}

impl<'a> IntoIterator for &'a OnelineBody {
    type Item = &'a Structure;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut OnelineBody {
    type Item = &'a mut Structure;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

decorate_impl! { Body, Attribute, Block }

span_impl! { Body, Attribute, Block }

forward_decorate_impl! {
    Structure => { Attribute, Block },
    BlockLabel => { Ident, String },
}

forward_span_impl! {
    Structure => { Attribute, Block },
    BlockLabel => { Ident, String }
}
