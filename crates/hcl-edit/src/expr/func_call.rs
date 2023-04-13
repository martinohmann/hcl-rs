use crate::expr::{Expression, IntoIter, Iter, IterMut};
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::{Ident, RawString};
use std::ops::Range;

#[derive(Debug, Clone, Eq)]
pub struct FuncCall {
    pub ident: Decorated<Ident>,
    pub args: FuncArgs,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl FuncCall {
    pub fn new(ident: Decorated<Ident>, args: FuncArgs) -> FuncCall {
        FuncCall {
            ident,
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
    pub fn new(args: Vec<Expression>) -> FuncArgs {
        FuncArgs {
            args,
            expand_final: false,
            trailing: RawString::default(),
            trailing_comma: false,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// An iterator visiting all values in insertion order. The iterator element type is `&'a
    /// Expression`.
    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.args.iter())
    }

    /// An iterator visiting all values in insertion order, with mutable references to the values.
    /// The iterator element type is `&'a mut Expression`.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.args.iter_mut())
    }

    pub fn expand_final(&self) -> bool {
        self.expand_final
    }

    pub fn set_expand_final(&mut self, yes: bool) {
        self.expand_final = yes;
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

impl<T> Extend<T> for FuncArgs
where
    T: Into<Expression>,
{
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = T>,
    {
        for v in iterable {
            self.args.push(v.into());
        }
    }
}

impl<T> FromIterator<T> for FuncArgs
where
    T: Into<Expression>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        FuncArgs::new(iter.into_iter().map(Into::into).collect())
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
