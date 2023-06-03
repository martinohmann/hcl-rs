use crate::expr::{Expression, IntoIter, Iter, IterMut};
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::{Ident, RawString};
use std::ops::Range;

/// Type representing a function call.
#[derive(Debug, Clone, Eq)]
pub struct FuncCall {
    /// The function identifier (or name).
    pub ident: Decorated<Ident>,
    /// The arguments between the function call's `(` and `)` argument delimiters.
    pub args: FuncArgs,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl FuncCall {
    /// Create a new `FuncCall` from an identifier and arguments.
    pub fn new(ident: impl Into<Decorated<Ident>>, args: FuncArgs) -> FuncCall {
        FuncCall {
            ident: ident.into(),
            args,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.ident.decor_mut().despan(input);
        self.args.despan(input);
    }
}

impl PartialEq for FuncCall {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident && self.args == other.args
    }
}

/// Type representing the arguments of a function call.
///
/// In the HCL grammar, function arguments are delimited by `(` and `)`.
#[derive(Debug, Clone, Eq, Default)]
pub struct FuncArgs {
    args: Vec<Expression>,
    expand_final: bool,
    pub(crate) trailing: RawString,
    trailing_comma: bool,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl FuncArgs {
    /// Constructs new, empty `FuncArgs`.
    #[inline]
    pub fn new() -> Self {
        FuncArgs::default()
    }

    /// Constructs new, empty `FuncArgs` with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        FuncArgs {
            args: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }

    /// Returns `true` if the function arguments are empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Returns the number of function arguments, also referred to as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        self.args.len()
    }

    /// Clears the function arguments.
    #[inline]
    pub fn clear(&mut self) {
        self.args.clear();
    }

    /// Returns a reference to the argument at the given index, or `None` if the index is out of
    /// bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Expression> {
        self.args.get(index)
    }

    /// Returns a mutable reference to the argument at the given index, or `None` if the index is
    /// out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Expression> {
        self.args.get_mut(index)
    }

    /// Inserts an argument at position `index`, shifting all arguments after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, arg: impl Into<Expression>) {
        self.args.insert(index, arg.into());
    }

    /// Removes the last argument and returns it, or [`None`] if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<Expression> {
        self.args.pop()
    }

    /// Appends an argument.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    #[inline]
    pub fn push(&mut self, arg: impl Into<Expression>) {
        self.args.push(arg.into());
    }

    /// Removes and returns the argument at position `index`, shifting all arguments after it to
    /// the left.
    ///
    /// Like `Vec::remove`, the argument is removed by shifting all of the arguments that follow
    /// it, preserving their relative order. **This perturbs the index of all of those elements!**
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Expression {
        self.args.remove(index)
    }

    /// An iterator visiting all values in insertion order. The iterator element type is `&'a
    /// Expression`.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.args.iter())
    }

    /// An iterator visiting all values in insertion order, with mutable references to the values.
    /// The iterator element type is `&'a mut Expression`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.args.iter_mut())
    }

    /// Returns `true` if the final argument is a `...` list expansion.
    #[inline]
    pub fn expand_final(&self) -> bool {
        self.expand_final
    }

    /// Set whether the final argument should be a `...` list expansion.
    #[inline]
    pub fn set_expand_final(&mut self, yes: bool) {
        self.expand_final = yes;
    }

    /// Return a reference to raw trailing decor before the function argument's closing `)`.
    #[inline]
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    /// Set the raw trailing decor before the function argument's closing `)`.
    #[inline]
    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    /// Returns `true` if the function arguments use a trailing comma.
    #[inline]
    pub fn trailing_comma(&self) -> bool {
        self.trailing_comma
    }

    /// Set whether the function arguments will use a trailing comma.
    #[inline]
    pub fn set_trailing_comma(&mut self, yes: bool) {
        self.trailing_comma = yes;
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        for arg in &mut self.args {
            arg.despan(input);
        }

        self.trailing.despan(input);
    }
}

impl PartialEq for FuncArgs {
    fn eq(&self, other: &Self) -> bool {
        self.args == other.args
            && self.trailing_comma == other.trailing_comma
            && self.trailing == other.trailing
    }
}

impl From<Vec<Expression>> for FuncArgs {
    fn from(args: Vec<Expression>) -> Self {
        FuncArgs {
            args,
            ..Default::default()
        }
    }
}

impl<T> Extend<T> for FuncArgs
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
        self.args.reserve(reserve);
        iter.for_each(|v| self.push(v));
    }
}

impl<T> FromIterator<T> for FuncArgs
where
    T: Into<Expression>,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let mut func_args = FuncArgs::with_capacity(lower);
        func_args.extend(iter);
        func_args
    }
}

impl IntoIterator for FuncArgs {
    type Item = Expression;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.args.into_iter())
    }
}

impl<'a> IntoIterator for &'a FuncArgs {
    type Item = &'a Expression;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut FuncArgs {
    type Item = &'a mut Expression;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

decorate_impl!(FuncCall, FuncArgs);
span_impl!(FuncCall, FuncArgs);
