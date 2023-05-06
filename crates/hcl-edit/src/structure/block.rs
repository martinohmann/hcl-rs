use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::structure::Body;
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
