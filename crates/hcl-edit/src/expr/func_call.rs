use crate::expr::{Expression, IntoIter, Iter, IterMut};
use crate::{Decor, Decorate, Decorated, Ident, RawString};
use std::ops::Range;

/// Type representing a (potentially namespaced) function name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncName {
    /// The function's namespace components, if any.
    pub namespace: Vec<Decorated<Ident>>,
    /// The function name.
    pub name: Decorated<Ident>,
}

impl FuncName {
    /// Create a new `FuncName` from a name identifier.
    pub fn new(name: impl Into<Decorated<Ident>>) -> FuncName {
        FuncName {
            namespace: Vec::new(),
            name: name.into(),
        }
    }

    /// Sets the function namespace from an iterator of namespace parts.
    pub fn set_namespace<I>(&mut self, namespace: I)
    where
        I: IntoIterator,
        I::Item: Into<Decorated<Ident>>,
    {
        self.namespace = namespace.into_iter().map(Into::into).collect();
    }

    /// Returns `true` if the function name is namespaced.
    ///
    /// ```
    /// use hcl_edit::{expr::FuncName, Ident};
    ///
    /// let mut func_name = FuncName::new(Ident::new("bar"));
    ///
    /// assert!(!func_name.is_namespaced());
    ///
    /// func_name.set_namespace([Ident::new("foo")]);
    ///
    /// assert!(func_name.is_namespaced());
    /// ```
    pub fn is_namespaced(&self) -> bool {
        !self.namespace.is_empty()
    }

    /// Returns `true` if the function has the given namespace.
    ///
    /// ```
    /// use hcl_edit::{expr::FuncName, Ident};
    ///
    /// let mut func_name = FuncName::new(Ident::new("baz"));
    ///
    /// assert!(!func_name.has_namespace(&["foo", "bar"]));
    ///
    /// func_name.set_namespace([Ident::new("foo"), Ident::new("bar")]);
    ///
    /// assert!(func_name.has_namespace(&["foo", "bar"]));
    /// assert!(!func_name.has_namespace(&["foo"]));
    /// assert!(!func_name.has_namespace(&["bar"]));
    /// ```
    pub fn has_namespace<T>(&self, namespace: &[T]) -> bool
    where
        T: AsRef<str>,
    {
        self.namespace.len() == namespace.len()
            && self
                .namespace
                .iter()
                .zip(namespace.iter())
                .all(|(a, b)| a.as_str() == b.as_ref())
    }

    pub(crate) fn despan(&mut self, input: &str) {
        for scope in &mut self.namespace {
            scope.decor_mut().despan(input);
        }
        self.name.decor_mut().despan(input);
    }
}

impl<T> From<T> for FuncName
where
    T: Into<Decorated<Ident>>,
{
    fn from(name: T) -> Self {
        FuncName {
            namespace: Vec::new(),
            name: name.into(),
        }
    }
}

impl<T, U> From<(T, U)> for FuncName
where
    T: IntoIterator,
    T::Item: Into<Decorated<Ident>>,
    U: Into<Decorated<Ident>>,
{
    fn from((namespace, name): (T, U)) -> Self {
        FuncName {
            namespace: namespace.into_iter().map(Into::into).collect(),
            name: name.into(),
        }
    }
}

/// Type representing a function call.
#[derive(Debug, Clone, Eq)]
pub struct FuncCall {
    /// The function name.
    pub name: FuncName,
    /// The arguments between the function call's `(` and `)` argument delimiters.
    pub args: FuncArgs,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl FuncCall {
    /// Create a new `FuncCall` from an identifier and arguments.
    pub fn new(name: impl Into<FuncName>, args: FuncArgs) -> FuncCall {
        FuncCall {
            name: name.into(),
            args,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.name.despan(input);
        self.args.despan(input);
    }
}

impl PartialEq for FuncCall {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.args == other.args
    }
}

/// Type representing the arguments of a function call.
///
/// In the HCL grammar, function arguments are delimited by `(` and `)`.
#[derive(Debug, Clone, Eq, Default)]
pub struct FuncArgs {
    args: Vec<Expression>,
    expand_final: bool,
    trailing: RawString,
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
