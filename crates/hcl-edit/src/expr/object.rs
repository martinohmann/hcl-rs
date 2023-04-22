use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::{Ident, RawString};
use std::ops::{self, Range};
use vecmap::map::{MutableKeys, VecMap};

/// An owning iterator over the entries of an `Object`.
///
/// Values of this type are created by the [`into_iter`] method on [`Object`] (provided by the
/// [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
/// [`IntoIterator`]: core::iter::IntoIterator
pub type ObjectIntoIter = Box<dyn Iterator<Item = (ObjectKey, ObjectValue)>>;

/// An iterator over the entries of an `Object`.
///
/// Values of this type are created by the [`iter`] method on [`Object`]. See its documentation for
/// more.
///
/// [`iter`]: Object::iter
pub type ObjectIter<'a> = Box<dyn Iterator<Item = (&'a ObjectKey, &'a ObjectValue)> + 'a>;

/// A mutable iterator over the entries of an `Object`.
///
/// Values of this type are created by the [`iter_mut`] method on [`Object`]. See its documentation
/// for more.
///
/// [`iter_mut`]: Object::iter_mut
pub type ObjectIterMut<'a> = Box<dyn Iterator<Item = (ObjectKeyMut<'a>, &'a mut ObjectValue)> + 'a>;

/// Type representing a HCL object.
#[derive(Debug, Clone, Eq, Default)]
pub struct Object {
    items: VecMap<ObjectKey, ObjectValue>,
    trailing: RawString,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl Object {
    /// Constructs a new, empty `Object`.
    #[inline]
    pub fn new() -> Self {
        Object::default()
    }

    /// Constructs a new, empty `Object` with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Object {
            items: VecMap::with_capacity(capacity),
            ..Default::default()
        }
    }

    /// Returns `true` if the object contains no items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items in the object, also referred to as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Clears the object, removing all items.
    #[inline]
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Return `true` if an equivalent to `key` exists in the object.
    #[inline]
    pub fn contains_key(&self, key: &ObjectKey) -> bool {
        self.items.contains_key(key)
    }

    /// Return a reference to the value stored for `key`, if it is present, else `None`.
    #[inline]
    pub fn get(&self, key: &ObjectKey) -> Option<&ObjectValue> {
        self.items.get(key)
    }

    /// Return a mutable reference to the value stored for `key`, if it is present, else `None`.
    #[inline]
    pub fn get_mut(&mut self, key: &ObjectKey) -> Option<&mut ObjectValue> {
        self.items.get_mut(key)
    }

    /// Return references to the key-value pair stored for `key`, if it is present, else `None`.
    #[inline]
    pub fn get_key_value(&self, key: &ObjectKey) -> Option<(&ObjectKey, &ObjectValue)> {
        self.items.get_key_value(key)
    }

    /// Return mutable references to the key-value pair stored for `key`, if it is present, else `None`.
    #[inline]
    pub fn get_key_value_mut<'a>(
        &'a mut self,
        key: &ObjectKey,
    ) -> Option<(ObjectKeyMut<'a>, &'a mut ObjectValue)> {
        self.items
            .get_full_mut2(key)
            .map(|(_, k, v)| (ObjectKeyMut::new(k), v))
    }

    /// Insert a key-value pair into the object.
    ///
    /// If an equivalent key already exists in the object: the key remains and retains in its place
    /// in the order, its corresponding value is updated with `value` and the older value is
    /// returned inside `Some(_)`.
    ///
    /// If no equivalent key existed in the object: the new key-value pair is inserted, last in
    /// order, and `None` is returned.
    #[inline]
    pub fn insert(
        &mut self,
        key: impl Into<ObjectKey>,
        value: impl Into<ObjectValue>,
    ) -> Option<ObjectValue> {
        self.items.insert(key.into(), value.into())
    }

    /// Remove the key-value pair equivalent to `key` and return its value.
    ///
    /// Like `Vec::remove`, the pair is removed by shifting all of the elements that follow it,
    /// preserving their relative order. **This perturbs the index of all of those elements!**
    #[inline]
    pub fn remove(&mut self, key: &ObjectKey) -> Option<ObjectValue> {
        self.items.remove(key)
    }

    /// Remove and return the key-value pair equivalent to `key`.
    ///
    /// Like `Vec::remove`, the pair is removed by shifting all of the elements that follow it,
    /// preserving their relative order. **This perturbs the index of all of those elements!**
    #[inline]
    pub fn remove_entry(&mut self, key: &ObjectKey) -> Option<(ObjectKey, ObjectValue)> {
        self.items.remove_entry(key)
    }

    /// An iterator visiting all key-value pairs in insertion order. The iterator element type is
    /// `(&'a ObjectKey, &'a ObjectValue)`.
    #[inline]
    pub fn iter(&self) -> ObjectIter<'_> {
        Box::new(self.items.iter())
    }

    /// An iterator visiting all key-value pairs in insertion order, with mutable references to the
    /// values. The iterator element type is `(ObjectKeyMut<'a>, &'a mut ObjectValue)`.
    #[inline]
    pub fn iter_mut(&mut self) -> ObjectIterMut<'_> {
        Box::new(
            self.items
                .iter_mut2()
                .map(|(k, v)| (ObjectKeyMut::new(k), v)),
        )
    }

    /// Return a reference to raw trailing decor before the object's closing `}`.
    #[inline]
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    /// Set the raw trailing decor before the object's closing `}`.
    #[inline]
    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.trailing.despan(input);

        for (key, value) in self.items.iter_mut2() {
            key.despan(input);
            value.despan(input);
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items && self.trailing == other.trailing
    }
}

impl From<VecMap<ObjectKey, ObjectValue>> for Object {
    fn from(items: VecMap<ObjectKey, ObjectValue>) -> Self {
        Object {
            items,
            ..Default::default()
        }
    }
}

impl<K, V> Extend<(K, V)> for Object
where
    K: Into<ObjectKey>,
    V: Into<ObjectValue>,
{
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iterable.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.items.reserve(reserve);
        iter.for_each(|(k, v)| {
            self.insert(k, v);
        });
    }
}

impl<K, V> FromIterator<(K, V)> for Object
where
    K: Into<ObjectKey>,
    V: Into<ObjectValue>,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let mut object = Object::with_capacity(lower);
        object.extend(iter);
        object
    }
}

impl IntoIterator for Object {
    type Item = (ObjectKey, ObjectValue);
    type IntoIter = ObjectIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.items.into_iter())
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = (&'a ObjectKey, &'a ObjectValue);
    type IntoIter = ObjectIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Object {
    type Item = (ObjectKeyMut<'a>, &'a mut ObjectValue);
    type IntoIter = ObjectIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Represents an object key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKey {
    /// Represents an unquoted identifier used as object key.
    Ident(Decorated<Ident>),
    /// Any valid HCL expression can be an object key.
    Expression(Expression),
}

impl ObjectKey {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            ObjectKey::Ident(ident) => ident.decor_mut().despan(input),
            ObjectKey::Expression(expr) => expr.despan(input),
        }
    }
}

impl From<Decorated<Ident>> for ObjectKey {
    fn from(ident: Decorated<Ident>) -> Self {
        ObjectKey::Ident(ident)
    }
}

impl From<Ident> for ObjectKey {
    fn from(ident: Ident) -> Self {
        ObjectKey::from(Decorated::new(ident))
    }
}

impl From<Expression> for ObjectKey {
    fn from(expr: Expression) -> Self {
        ObjectKey::Expression(expr)
    }
}

/// Allows mutable access to the surrounding [`Decor`](crate::repr::Decor) of an [`ObjectKey`] but
/// not to its value.
///
/// This type wraps the object key in the iterator returned by [`Object::iter_mut`].
#[derive(Debug, Eq, PartialEq)]
pub struct ObjectKeyMut<'k> {
    key: &'k mut ObjectKey,
}

impl<'k> ObjectKeyMut<'k> {
    pub(crate) fn new(key: &'k mut ObjectKey) -> ObjectKeyMut<'k> {
        ObjectKeyMut { key }
    }

    /// Returns an immutable reference to the wrapped `ObjectKey`.
    pub fn get(&self) -> &ObjectKey {
        self.key
    }
}

impl<'k> ops::Deref for ObjectKeyMut<'k> {
    type Target = ObjectKey;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'k> Decorate for ObjectKeyMut<'k> {
    fn decor(&self) -> &Decor {
        self.key.decor()
    }

    fn decor_mut(&mut self) -> &mut Decor {
        self.key.decor_mut()
    }
}

impl<'k> Span for ObjectKeyMut<'k> {
    fn span(&self) -> Option<Range<usize>> {
        self.key.span()
    }
}

/// Represents the assignment operator between an object key and its value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ObjectValueAssignment {
    /// Colon (`:`) assignment operator.
    Colon,
    /// Equals (`=`) assignment operator.
    #[default]
    Equals,
}

/// Represents the character that terminates an object value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ObjectValueTerminator {
    /// No terminator.
    None,
    /// Newline terminated.
    Newline,
    /// Comma terminated.
    #[default]
    Comma,
}

/// Represents an object value together with it's assignment operator and value terminator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectValue {
    expr: Expression,
    assignment: ObjectValueAssignment,
    terminator: ObjectValueTerminator,
}

impl ObjectValue {
    /// Creates a new `ObjectValue` for an expression.
    pub fn new(expr: Expression) -> ObjectValue {
        ObjectValue {
            expr,
            assignment: ObjectValueAssignment::default(),
            terminator: ObjectValueTerminator::default(),
        }
    }

    /// Returns a reference to the object value's [`Expression`].
    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    /// Returns a mutable reference to the object value's [`Expression`].
    pub fn expr_mut(&mut self) -> &mut Expression {
        &mut self.expr
    }

    /// Converts the object value into an [`Expression`].
    pub fn into_expr(self) -> Expression {
        self.expr
    }

    /// Returns the object value assignment operator.
    pub fn assignment(&self) -> ObjectValueAssignment {
        self.assignment
    }

    /// Sets the object value assignment operator.
    pub fn set_assignment(&mut self, sep: ObjectValueAssignment) {
        self.assignment = sep;
    }

    /// Returns the object value terminator.
    pub fn terminator(&self) -> ObjectValueTerminator {
        self.terminator
    }

    /// Sets the object value terminator.
    pub fn set_terminator(&mut self, terminator: ObjectValueTerminator) {
        self.terminator = terminator;
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.expr.despan(input);
    }
}

impl From<Expression> for ObjectValue {
    fn from(expr: Expression) -> Self {
        ObjectValue::new(expr)
    }
}

decorate_impl!(Object);
span_impl!(Object);
forward_decorate_impl!(ObjectKey => { Ident, Expression });
forward_span_impl!(ObjectKey => { Ident, Expression });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Array;
    use pretty_assertions::assert_eq;

    #[test]
    fn object_access() {
        // Ident key.
        let mut obj = Object::new();
        let mut key = ObjectKey::from(Ident::new_unchecked("foo"));
        key.decorate(("/* prefix */", "/* suffix */"));

        let value = ObjectValue::from(Expression::from("bar"));

        obj.insert(key.clone(), value.clone());

        assert_eq!(obj.get(&key), Some(&value));

        key.decor_mut().clear();

        assert_eq!(obj.get(&key), Some(&value));

        let (key, _) = obj.remove_entry(&key).unwrap();
        assert_eq!(key.decor().prefix(), Some(&RawString::from("/* prefix */")));
        assert_eq!(key.decor().suffix(), Some(&RawString::from("/* suffix */")));

        // Expression key.
        let mut array = Array::new();
        array.push("foo");
        let mut key = ObjectKey::from(Expression::from(array));
        key.decorate(("/* prefix */", "/* suffix */"));

        let value = ObjectValue::from(Expression::from("bar"));

        obj.insert(key.clone(), value.clone());

        assert_eq!(obj.get(&key), Some(&value));

        key.decor_mut().clear();

        assert_eq!(obj.get(&key), Some(&value));

        let (key, _) = obj.remove_entry(&key).unwrap();
        assert_eq!(key.decor().prefix(), Some(&RawString::from("/* prefix */")));
        assert_eq!(key.decor().suffix(), Some(&RawString::from("/* suffix */")));
    }
}
