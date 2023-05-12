use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::Ident;
use std::ops::Range;

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
    pub fn new(key: impl Into<Decorated<Ident>>, value: impl Into<Expression>) -> Attribute {
        Attribute {
            key: key.into(),
            value: value.into(),
            decor: Decor::default(),
            span: None,
        }
    }

    /// Returns `true` if the attribute has the given key.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::{structure::Attribute, Ident};
    ///
    /// let attr = Attribute::new(Ident::new("foo"), "bar");
    /// assert!(attr.has_key("foo"));
    /// assert!(!attr.has_key("bar"));
    /// ```
    #[inline]
    pub fn has_key(&self, key: &str) -> bool {
        self.key.as_str() == key
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

decorate_impl!(Attribute);
span_impl!(Attribute);
