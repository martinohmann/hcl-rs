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

/// Type representing a HCL array.
#[derive(Debug, Clone, Eq, Default)]
pub struct Array {
    values: Vec<Expression>,
    pub(crate) trailing: RawString,
    trailing_comma: bool,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl Array {
    /// Constructs a new, empty `Array`.
    #[inline]
    pub fn new() -> Self {
        Array::default()
    }

    /// Constructs a new, empty `Array` with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Array {
            values: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }

    /// Returns `true` if the array contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns the number of elements in the array, also referred to as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Clears the array, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Returns a reference to the value at the given index, or `None` if the index is out of
    /// bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Expression> {
        self.values.get(index)
    }

    /// Returns a mutable reference to the value at the given index, or `None` if the index is out
    /// of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Expression> {
        self.values.get_mut(index)
    }

    /// Inserts an element at position `index` within the array, shifting all elements after it to
    /// the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, value: impl Into<Expression>) {
        self.values.insert(index, value.into());
    }

    /// Appends an element to the back of the array.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    #[inline]
    pub fn push(&mut self, value: impl Into<Expression>) {
        self.values.push(value.into());
    }

    /// Removes the last element from the array and returns it, or [`None`] if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<Expression> {
        self.values.pop()
    }

    /// Removes and returns the element at position `index` within the array, shifting all elements
    /// after it to the left.
    ///
    /// Like `Vec::remove`, the element is removed by shifting all of the elements that follow it,
    /// preserving their relative order. **This perturbs the index of all of those elements!**
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Expression {
        self.values.remove(index)
    }

    /// An iterator visiting all values in insertion order. The iterator element type is `&'a
    /// Expression`.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.values.iter())
    }

    /// An iterator visiting all values in insertion order, with mutable references to the values.
    /// The iterator element type is `&'a mut Expression`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.values.iter_mut())
    }

    /// Return a reference to raw trailing decor before the array's closing `]`.
    #[inline]
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    /// Set the raw trailing decor before the array's closing `]`.
    #[inline]
    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    /// Returns `true` if the array uses a trailing comma.
    #[inline]
    pub fn trailing_comma(&self) -> bool {
        self.trailing_comma
    }

    /// Set whether the array will use a trailing comma.
    #[inline]
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
            ..Default::default()
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
        let iter = iterable.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.values.reserve(reserve);
        iter.for_each(|v| self.push(v));
    }
}

impl<T> FromIterator<T> for Array
where
    T: Into<Expression>,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let mut array = Array::with_capacity(lower);
        array.extend(iter);
        array
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
