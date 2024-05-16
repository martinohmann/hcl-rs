use crate::expr::Expression;
use crate::format::{Format, Formatter};
use crate::visit_mut::VisitMut;
use crate::{Decor, Decorate, Decorated, Ident, Span};
use std::ops::{self, Range};

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

impl Format for Attribute {
    fn format(&mut self, fmt: &mut Formatter) {
        fmt.visit_attr_mut(AttributeMut::new(self));
        fmt.reset();
    }
}

/// Allows mutable access to the value and surrounding [`Decor`] of an [`Attribute`] but not to its
/// key.
///
/// This type wraps the attribute returned by
/// [`Body::get_attribute_mut`](crate::structure::Body::get_attribute_mut) and in the iterator
/// returned by [`Body::attributes_mut`](crate::structure::Body::attributes_mut).
pub struct AttributeMut<'a> {
    attr: &'a mut Attribute,
}

impl<'a> AttributeMut<'a> {
    pub(crate) fn new(attr: &'a mut Attribute) -> AttributeMut<'a> {
        AttributeMut { attr }
    }

    /// Returns an immutable reference to the wrapped `Attribute`.
    pub fn get(&self) -> &Attribute {
        self.attr
    }

    /// Returns a mutable reference to the wrapped `Attribute`'s key's decor.
    pub fn key_decor_mut(&mut self) -> &mut Decor {
        self.attr.key.decor_mut()
    }

    /// Returns a mutable reference to the wrapped `Attribute`'s value.
    pub fn value_mut(&mut self) -> &mut Expression {
        &mut self.attr.value
    }
}

impl<'a> ops::Deref for AttributeMut<'a> {
    type Target = Attribute;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a> Decorate for AttributeMut<'a> {
    fn decor(&self) -> &Decor {
        self.attr.decor()
    }

    fn decor_mut(&mut self) -> &mut Decor {
        self.attr.decor_mut()
    }
}

impl<'a> Span for AttributeMut<'a> {
    fn span(&self) -> Option<Range<usize>> {
        self.attr.span()
    }
}

impl<'a> Format for AttributeMut<'a> {
    fn format(&mut self, fmt: &mut Formatter) {
        self.attr.format(fmt);
    }
}
