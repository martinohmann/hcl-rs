#![allow(missing_docs)]

use crate::expr::Expression;
use crate::repr::{Decor, Decorate, SetSpan, Span};
use crate::RawString;
use std::ops::Range;

/// An owning iterator over the values of an `Array`.
///
/// Values of this type are created by the [`into_iter`] method on [`Array`] (provided by the
/// [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
/// [`IntoIterator`]: core::iter::IntoIterator
pub type IntoIter = Box<dyn Iterator<Item = Expression>>;

/// An iterator over the values of an `Array`.
///
/// Values of this type are created by the [`iter`] method on [`Array`]. See its documentation for
/// more.
///
/// [`iter`]: Array::iter
pub type Iter<'a> = Box<dyn Iterator<Item = &'a Expression> + 'a>;

/// A mutable iterator over the values of an `Array`.
///
/// Values of this type are created by the [`iter_mut`] method on [`Array`]. See its documentation
/// for more.
///
/// [`iter_mut`]: Array::iter_mut
pub type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Expression> + 'a>;

#[derive(Debug, Clone, Eq, Default)]
pub struct Array {
    values: Vec<Expression>,
    trailing: RawString,
    trailing_comma: bool,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl Array {
    pub fn new() -> Array {
        Array {
            values: Vec::new(),
            trailing: RawString::default(),
            trailing_comma: false,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Expression> {
        self.values.get(index)
    }

    pub fn insert(&mut self, index: usize, value: impl Into<Expression>) {
        self.values.insert(index, value.into());
    }

    pub fn push(&mut self, value: impl Into<Expression>) {
        self.values.push(value.into());
    }

    pub fn remove(&mut self, index: usize) -> Expression {
        self.values.remove(index)
    }

    /// An iterator visiting all values in insertion order. The iterator element type is `&'a
    /// Expression`.
    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.values.iter())
    }

    /// An iterator visiting all values in insertion order, with mutable references to the values.
    /// The iterator element type is `&'a mut Expression`.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.values.iter_mut())
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    pub fn trailing_comma(&self) -> bool {
        self.trailing_comma
    }

    pub fn set_trailing_comma(&mut self, yes: bool) {
        self.trailing_comma = yes;
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.trailing.despan(input);

        for value in &mut self.values {
            value.despan(input);
        }
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
            && self.trailing_comma == other.trailing_comma
            && self.trailing == other.trailing
    }
}

impl From<Vec<Expression>> for Array {
    fn from(values: Vec<Expression>) -> Self {
        Array {
            values,
            trailing: RawString::default(),
            trailing_comma: false,
            decor: Decor::default(),
            span: None,
        }
    }
}

impl<T> Extend<T> for Array
where
    T: Into<Expression>,
{
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = T>,
    {
        for v in iterable {
            self.push(v);
        }
    }
}

impl<T> FromIterator<T> for Array
where
    T: Into<Expression>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().map(Into::into).collect::<Vec<_>>().into()
    }
}

impl IntoIterator for Array {
    type Item = Expression;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.values.into_iter())
    }
}

impl<'a> IntoIterator for &'a Array {
    type Item = &'a Expression;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Array {
    type Item = &'a mut Expression;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

decorate_impl!(Array);
span_impl!(Array);
