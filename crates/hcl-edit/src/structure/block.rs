use crate::repr::{Decor, Decorate, Decorated};
use crate::structure::{Attribute, Body, Structure};
use crate::Ident;
use std::ops::{self, Range};

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
    pub body: Body,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl Block {
    /// Creates a new `Block` from an identifier.
    pub fn new(ident: impl Into<Decorated<Ident>>) -> Block {
        Block {
            ident: ident.into(),
            labels: Vec::new(),
            body: Body::new(),
            decor: Decor::default(),
            span: None,
        }
    }

    /// Creates a new [`BlockBuilder`] to start building a new `Block` with the provided
    /// identifier.
    #[inline]
    pub fn builder(ident: impl Into<Decorated<Ident>>) -> BlockBuilder {
        BlockBuilder::new(ident.into())
    }

    /// Returns `true` if the block has labels.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::{structure::Block, Ident};
    ///
    /// let block = Block::new(Ident::new("foo"));
    /// assert!(!block.is_labeled());
    ///
    /// let labeled_block = Block::builder(Ident::new("foo"))
    ///     .label("bar")
    ///     .build();
    /// assert!(labeled_block.is_labeled());
    /// ```
    #[inline]
    pub fn is_labeled(&self) -> bool {
        !self.labels.is_empty()
    }

    /// Returns `true` if the block has the given identifier.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::{structure::Block, Ident};
    ///
    /// let block = Block::new(Ident::new("foo"));
    /// assert!(block.has_ident("foo"));
    /// assert!(!block.has_ident("bar"));
    /// ```
    #[inline]
    pub fn has_ident(&self, ident: &str) -> bool {
        self.ident.as_str() == ident
    }

    /// Returns `true` if the `Block`'s labels and the provided ones share a common prefix.
    ///
    /// For example, `&["foo"]` will match blocks that fulfil either of these criteria:
    ///
    /// - Single `"foo"` label.
    /// - Multiple labels, with `"foo"` being in first position.
    ///
    /// For an alternative which matches labels exactly see [`Block::has_exact_labels`].
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl_edit::{structure::Block, Ident};
    ///
    /// let block = Block::builder(Ident::new("resource"))
    ///     .labels(["aws_s3_bucket", "mybucket"])
    ///     .build();
    ///
    /// assert!(block.has_labels(&["aws_s3_bucket"]));
    /// assert!(block.has_labels(&["aws_s3_bucket", "mybucket"]));
    /// assert!(!block.has_labels(&["mybucket"]));
    /// ```
    ///
    /// One use case for this method is to find blocks in a [`Body`] that have a common label
    /// prefix:
    ///
    /// ```
    /// use hcl_edit::structure::{Attribute, Block, Body};
    /// use hcl_edit::Ident;
    ///
    /// let body = Body::builder()
    ///     .attribute(Attribute::new(Ident::new("foo"), "bar"))
    ///     .block(
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket1"])
    ///     )
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
    /// let buckets: Vec<&Block> = body.get_blocks("resource")
    ///     .filter(|block| block.has_labels(&["aws_s3_bucket"]))
    ///     .collect();
    ///
    /// assert_eq!(
    ///     buckets,
    ///     [
    ///         &Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket1"])
    ///             .build(),
    ///         &Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket2"])
    ///             .build()
    ///     ]
    /// );
    /// ```
    pub fn has_labels<T>(&self, labels: &[T]) -> bool
    where
        T: AsRef<str>,
    {
        if self.labels.len() < labels.len() {
            false
        } else {
            self.labels
                .iter()
                .zip(labels.iter())
                .all(|(a, b)| a.as_str() == b.as_ref())
        }
    }

    /// Returns `true` if the `Block`'s labels match the provided ones exactly.
    ///
    /// For an alternative which matches a common label prefix see [`Block::has_labels`].
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl_edit::{structure::Block, Ident};
    ///
    /// let block = Block::builder(Ident::new("resource"))
    ///     .labels(["aws_s3_bucket", "mybucket"])
    ///     .build();
    ///
    /// assert!(!block.has_exact_labels(&["aws_s3_bucket"]));
    /// assert!(block.has_exact_labels(&["aws_s3_bucket", "mybucket"]));
    /// ```
    ///
    /// One use case for this method is to find blocks in a [`Body`] that have an exact set of
    /// labels:
    ///
    /// ```
    /// use hcl_edit::structure::{Attribute, Block, Body};
    /// use hcl_edit::Ident;
    ///
    /// let body = Body::builder()
    ///     .block(
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket1"])
    ///     )
    ///     .block(
    ///         Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket2"])
    ///     )
    ///     .build();
    ///
    /// let buckets: Vec<&Block> = body.get_blocks("resource")
    ///     .filter(|block| block.has_exact_labels(&["aws_s3_bucket", "bucket1"]))
    ///     .collect();
    ///
    /// assert_eq!(
    ///     buckets,
    ///     [
    ///         &Block::builder(Ident::new("resource"))
    ///             .labels(["aws_s3_bucket", "bucket1"])
    ///             .build(),
    ///     ]
    /// );
    /// ```
    pub fn has_exact_labels<T>(&self, labels: &[T]) -> bool
    where
        T: AsRef<str>,
    {
        self.labels.len() == labels.len() && self.has_labels(labels)
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

decorate_impl!(Block);
span_impl!(Block);
forward_decorate_impl!(BlockLabel => { Ident, String });
forward_span_impl!(BlockLabel => { Ident, String });

/// `BlockBuilder` builds an HCL [`Block`].
///
/// The builder allows to build the `Block` by adding labels, attributes and other nested blocks
/// via chained method calls. A call to [`.build()`](BlockBuilder::build) produces the final
/// `Block`.
///
/// ## Example
///
/// ```
/// use hcl_edit::structure::{Attribute, Block, Body};
/// use hcl_edit::Ident;
///
/// let block = Block::builder(Ident::new("resource"))
///     .labels(["aws_s3_bucket", "mybucket"])
///     .attribute(Attribute::new(Ident::new("name"), "mybucket"))
///     .block(
///         Block::builder(Ident::new("logging"))
///             .attribute(Attribute::new(Ident::new("target_bucket"), "mylogsbucket"))
///     )
///     .build();
/// ```
#[derive(Debug)]
pub struct BlockBuilder {
    ident: Decorated<Ident>,
    labels: Vec<BlockLabel>,
    body: Body,
}

impl BlockBuilder {
    fn new(ident: Decorated<Ident>) -> BlockBuilder {
        BlockBuilder {
            ident,
            labels: Vec::new(),
            body: Body::new(),
        }
    }

    /// Adds a `BlockLabel`.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    #[inline]
    pub fn label(mut self, label: impl Into<BlockLabel>) -> Self {
        self.labels.push(label.into());
        self
    }

    /// Adds `BlockLabel`s from an iterator.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    #[inline]
    pub fn labels<I>(mut self, iter: I) -> BlockBuilder
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
    #[inline]
    pub fn attribute(self, attr: impl Into<Attribute>) -> BlockBuilder {
        self.structure(attr.into())
    }

    /// Adds `Attribute`s to the block body from an iterator.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    #[inline]
    pub fn attributes<I>(self, iter: I) -> BlockBuilder
    where
        I: IntoIterator,
        I::Item: Into<Attribute>,
    {
        self.structures(iter.into_iter().map(Into::into))
    }

    /// Adds another `Block` to the block body.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    #[inline]
    pub fn block(self, block: impl Into<Block>) -> BlockBuilder {
        self.structure(block.into())
    }

    /// Adds `Block`s to the block body from an iterator.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    #[inline]
    pub fn blocks<I>(self, iter: I) -> BlockBuilder
    where
        I: IntoIterator,
        I::Item: Into<Block>,
    {
        self.structures(iter.into_iter().map(Into::into))
    }

    /// Adds a `Structure` to the block body.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    #[inline]
    pub fn structure(mut self, structures: impl Into<Structure>) -> BlockBuilder {
        self.body.push(structures.into());
        self
    }

    /// Adds `Structure`s to the block body from an iterator.
    ///
    /// Consumes `self` and returns a new `BlockBuilder`.
    #[inline]
    pub fn structures<I>(mut self, iter: I) -> BlockBuilder
    where
        I: IntoIterator,
        I::Item: Into<Structure>,
    {
        self.body.extend(iter);
        self
    }

    /// Consumes `self` and builds the [`Block`] from the items added via the builder methods.
    #[inline]
    pub fn build(self) -> Block {
        Block {
            ident: self.ident,
            labels: self.labels,
            body: self.body,
            decor: Decor::default(),
            span: None,
        }
    }
}

impl From<BlockBuilder> for Block {
    #[inline]
    fn from(builder: BlockBuilder) -> Self {
        builder.build()
    }
}
