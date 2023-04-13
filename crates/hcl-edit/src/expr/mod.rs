//! Types to represent the HCL expression sub-language.

#![allow(missing_docs)]

use crate::encode::{EncodeDecorated, EncodeState, NO_DECOR};
use crate::repr::{Decor, Decorate, Decorated, Formatted, SetSpan, Span, Spanned};
use crate::template::{HeredocTemplate, StringTemplate};
use crate::{parser, Ident, Number, RawString};
use std::fmt;
use std::ops::{self, Range};
use std::str::FromStr;
use vecmap::map::{MutableKeys, VecMap};

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::expr::{BinaryOperator, UnaryOperator};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Null(Decorated<Null>),
    Bool(Decorated<bool>),
    Number(Formatted<Number>),
    String(Decorated<String>),
    Array(Array),
    Object(Object),
    Template(StringTemplate),
    HeredocTemplate(Box<HeredocTemplate>),
    Parenthesis(Box<Parenthesis>),
    Variable(Decorated<Ident>),
    Conditional(Box<Conditional>),
    FuncCall(Box<FuncCall>),
    Traversal(Box<Traversal>),
    UnaryOp(Box<UnaryOp>),
    BinaryOp(Box<BinaryOp>),
    ForExpr(Box<ForExpr>),
}

impl Expression {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            Expression::Null(n) => n.decor_mut().despan(input),
            Expression::Bool(b) => b.decor_mut().despan(input),
            Expression::Number(n) => n.decor_mut().despan(input),
            Expression::String(s) => s.decor_mut().despan(input),
            Expression::Array(array) => array.despan(input),
            Expression::Object(object) => object.despan(input),
            Expression::Template(template) => template.despan(input),
            Expression::HeredocTemplate(heredoc) => heredoc.despan(input),
            Expression::Parenthesis(expr) => expr.despan(input),
            Expression::Variable(var) => var.decor_mut().despan(input),
            Expression::ForExpr(expr) => expr.despan(input),
            Expression::Conditional(cond) => cond.despan(input),
            Expression::FuncCall(call) => call.despan(input),
            Expression::UnaryOp(op) => op.despan(input),
            Expression::BinaryOp(op) => op.despan(input),
            Expression::Traversal(traversal) => traversal.despan(input),
        }
    }
}

impl FromStr for Expression {
    type Err = parser::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse_expr(s)
    }
}

impl From<&str> for Expression {
    fn from(s: &str) -> Self {
        Expression::from(String::from(s))
    }
}

impl From<String> for Expression {
    fn from(s: String) -> Self {
        Expression::from(Decorated::new(s))
    }
}

impl From<Decorated<String>> for Expression {
    fn from(s: Decorated<String>) -> Self {
        Expression::String(s)
    }
}

impl From<Array> for Expression {
    fn from(array: Array) -> Self {
        Expression::Array(array)
    }
}

impl From<Object> for Expression {
    fn from(object: Object) -> Self {
        Expression::Object(object)
    }
}

impl From<Parenthesis> for Expression {
    fn from(value: Parenthesis) -> Self {
        Expression::Parenthesis(Box::new(value))
    }
}

impl From<Traversal> for Expression {
    fn from(traversal: Traversal) -> Self {
        Expression::Traversal(Box::new(traversal))
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode_decorated(&mut state, NO_DECOR)
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Parenthesis {
    inner: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl Parenthesis {
    pub fn new(inner: Expression) -> Parenthesis {
        Parenthesis {
            inner,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn inner(&self) -> &Expression {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut Expression {
        &mut self.inner
    }

    pub fn into_inner(self) -> Expression {
        self.inner
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.inner.despan(input);
    }
}

impl PartialEq for Parenthesis {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[derive(Debug, Clone, Eq, Default)]
pub struct Array {
    values: Vec<Expression>,
    pub(crate) trailing: RawString,
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

#[derive(Debug, Clone, Eq, Default)]
pub struct Object {
    items: VecMap<ObjectKey, ObjectValue>,
    pub(crate) trailing: RawString,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl Object {
    pub fn new() -> Object {
        Object {
            items: VecMap::new(),
            trailing: RawString::default(),
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn contains_key(&self, key: &ObjectKey) -> bool {
        self.items.contains_key(key)
    }

    pub fn get(&self, key: &ObjectKey) -> Option<&ObjectValue> {
        self.items.get(key)
    }

    pub fn insert(
        &mut self,
        key: impl Into<ObjectKey>,
        value: impl Into<ObjectValue>,
    ) -> Option<ObjectValue> {
        self.items.insert(key.into(), value.into())
    }

    pub fn remove(&mut self, key: &ObjectKey) -> Option<ObjectValue> {
        self.items.remove(key)
    }

    pub fn remove_entry(&mut self, key: &ObjectKey) -> Option<(ObjectKey, ObjectValue)> {
        self.items.remove_entry(key)
    }

    /// An iterator visiting all key-value pairs in insertion order. The iterator element type is
    /// `(&'a ObjectKey, &'a ObjectValue)`.
    pub fn iter(&self) -> ObjectIter<'_> {
        Box::new(self.items.iter())
    }

    /// An iterator visiting all key-value pairs in insertion order, with mutable references to the
    /// values. The iterator element type is `(ObjectKeyMut<'a>, &'a mut ObjectValue)`.
    pub fn iter_mut(&mut self) -> ObjectIterMut<'_> {
        Box::new(
            self.items
                .iter_mut2()
                .map(|(k, v)| (ObjectKeyMut::new(k), v)),
        )
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

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
            trailing: RawString::default(),
            decor: Decor::default(),
            span: None,
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
        for (k, v) in iterable {
            self.insert(k, v);
        }
    }
}

impl<K, V> FromIterator<(K, V)> for Object
where
    K: Into<ObjectKey>,
    V: Into<ObjectValue>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        iter.into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect::<VecMap<_, _>>()
            .into()
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKey {
    Ident(Decorated<Ident>),
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ObjectValueAssignment {
    Colon,
    #[default]
    Equals,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ObjectValueTerminator {
    None,
    Newline,
    #[default]
    Comma,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectValue {
    expr: Expression,
    assignment: ObjectValueAssignment,
    terminator: ObjectValueTerminator,
}

impl ObjectValue {
    pub fn new(expr: Expression) -> ObjectValue {
        ObjectValue {
            expr,
            assignment: ObjectValueAssignment::default(),
            terminator: ObjectValueTerminator::default(),
        }
    }

    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    pub fn expr_mut(&mut self) -> &mut Expression {
        &mut self.expr
    }

    pub fn into_expr(self) -> Expression {
        self.expr
    }

    pub fn assignment(&self) -> ObjectValueAssignment {
        self.assignment
    }

    pub fn set_assignment(&mut self, sep: ObjectValueAssignment) {
        self.assignment = sep;
    }

    pub fn terminator(&self) -> ObjectValueTerminator {
        self.terminator
    }

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

#[derive(Debug, Clone, Eq)]
pub struct Conditional {
    pub cond_expr: Expression,
    pub true_expr: Expression,
    pub false_expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl Conditional {
    pub fn new(
        cond_expr: Expression,
        true_expr: Expression,
        false_expr: Expression,
    ) -> Conditional {
        Conditional {
            cond_expr,
            true_expr,
            false_expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.cond_expr.despan(input);
        self.true_expr.despan(input);
        self.false_expr.despan(input);
    }
}

impl PartialEq for Conditional {
    fn eq(&self, other: &Self) -> bool {
        self.cond_expr == other.cond_expr
            && self.true_expr == other.true_expr
            && self.false_expr == other.false_expr
    }
}

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
    pub(crate) trailing: RawString,
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

#[derive(Debug, Clone, Eq)]
pub struct Traversal {
    pub expr: Expression,
    pub operators: Vec<Decorated<TraversalOperator>>,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl Traversal {
    pub fn new(expr: Expression, operators: Vec<Decorated<TraversalOperator>>) -> Traversal {
        Traversal {
            expr,
            operators,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);

        for operator in &mut self.operators {
            operator.despan(input);
        }
    }
}

impl PartialEq for Traversal {
    fn eq(&self, other: &Self) -> bool {
        self.expr == other.expr && self.operators == other.operators
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Null;

impl fmt::Display for Null {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "null")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Splat;

impl fmt::Display for Splat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "*")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalOperator {
    AttrSplat(Decorated<Splat>),
    FullSplat(Decorated<Splat>),
    GetAttr(Decorated<Ident>),
    Index(Expression),
    LegacyIndex(Decorated<u64>),
}

impl TraversalOperator {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            TraversalOperator::AttrSplat(splat) | TraversalOperator::FullSplat(splat) => {
                splat.decor_mut().despan(input);
            }
            TraversalOperator::GetAttr(ident) => ident.decor_mut().despan(input),
            TraversalOperator::Index(expr) => expr.despan(input),
            TraversalOperator::LegacyIndex(index) => index.decor_mut().despan(input),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct UnaryOp {
    pub operator: Spanned<UnaryOperator>,
    pub expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl UnaryOp {
    pub fn new(operator: Spanned<UnaryOperator>, expr: Expression) -> UnaryOp {
        UnaryOp {
            operator,
            expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);
    }
}

impl PartialEq for UnaryOp {
    fn eq(&self, other: &Self) -> bool {
        self.operator == other.operator && self.expr == other.expr
    }
}

#[derive(Debug, Clone, Eq)]
pub struct BinaryOp {
    pub lhs_expr: Expression,
    pub operator: Spanned<BinaryOperator>,
    pub rhs_expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl BinaryOp {
    pub fn new(
        lhs_expr: Expression,
        operator: Spanned<BinaryOperator>,
        rhs_expr: Expression,
    ) -> BinaryOp {
        BinaryOp {
            lhs_expr,
            operator,
            rhs_expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.lhs_expr.despan(input);
        self.rhs_expr.despan(input);
    }
}

impl PartialEq for BinaryOp {
    fn eq(&self, other: &Self) -> bool {
        self.lhs_expr == other.lhs_expr
            && self.operator == other.operator
            && self.rhs_expr == other.rhs_expr
    }
}

#[derive(Debug, Clone, Eq)]
pub struct ForExpr {
    pub intro: ForIntro,
    pub key_expr: Option<Expression>,
    pub value_expr: Expression,
    pub grouping: bool,
    pub cond: Option<ForCond>,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl ForExpr {
    pub fn new(intro: ForIntro, value_expr: Expression) -> ForExpr {
        ForExpr {
            intro,
            key_expr: None,
            value_expr,
            grouping: false,
            cond: None,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.intro.despan(input);

        if let Some(key_expr) = &mut self.key_expr {
            key_expr.despan(input);
        }

        self.value_expr.despan(input);

        if let Some(cond) = &mut self.cond {
            cond.despan(input);
        }
    }
}

impl PartialEq for ForExpr {
    fn eq(&self, other: &Self) -> bool {
        self.intro == other.intro
            && self.key_expr == other.key_expr
            && self.value_expr == other.value_expr
            && self.grouping == other.grouping
            && self.cond == other.cond
    }
}

#[derive(Debug, Clone, Eq)]
pub struct ForIntro {
    pub key_var: Option<Decorated<Ident>>,
    pub value_var: Decorated<Ident>,
    pub collection_expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl ForIntro {
    pub fn new(value_var: Decorated<Ident>, collection_expr: Expression) -> ForIntro {
        ForIntro {
            key_var: None,
            value_var,
            collection_expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        if let Some(key_var) = &mut self.key_var {
            key_var.decor_mut().despan(input);
        }

        self.value_var.decor_mut().despan(input);
        self.collection_expr.despan(input);
    }
}

impl PartialEq for ForIntro {
    fn eq(&self, other: &Self) -> bool {
        self.key_var == other.key_var
            && self.value_var == other.value_var
            && self.collection_expr == other.collection_expr
    }
}

#[derive(Debug, Clone, Eq)]
pub struct ForCond {
    pub expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl ForCond {
    pub fn new(expr: Expression) -> ForCond {
        ForCond {
            expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);
    }
}

impl PartialEq for ForCond {
    fn eq(&self, other: &Self) -> bool {
        self.expr == other.expr
    }
}

impl From<Expression> for ForCond {
    fn from(value: Expression) -> Self {
        ForCond::new(value)
    }
}

decorate_impl! {
    Array, Object, Parenthesis, FuncCall, FuncArgs, Conditional,
    Traversal, UnaryOp, BinaryOp, ForExpr, ForIntro, ForCond
}

span_impl! {
    Array, Object, Parenthesis, FuncCall, FuncArgs, Conditional,
    Traversal, UnaryOp, BinaryOp, ForExpr, ForIntro, ForCond
}

forward_decorate_impl! {
    Expression => {
        Null, Bool, Number, String, Array, Object, Template, HeredocTemplate, Parenthesis,
        Variable, ForExpr, Conditional, FuncCall, UnaryOp, BinaryOp, Traversal
    },
    TraversalOperator => { AttrSplat, FullSplat, GetAttr, Index, LegacyIndex },
    ObjectKey => { Ident, Expression }
}

forward_span_impl! {
    Expression => {
        Null, Bool, Number, String, Array, Object, Template, HeredocTemplate, Parenthesis,
        Variable, ForExpr, Conditional, FuncCall, UnaryOp, BinaryOp, Traversal
    },
    TraversalOperator => { AttrSplat, FullSplat, GetAttr, Index, LegacyIndex },
    ObjectKey => { Ident, Expression }
}

#[cfg(test)]
mod tests {
    use super::*;
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
