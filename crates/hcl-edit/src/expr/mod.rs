use crate::encode::{EncodeDecorated, EncodeState, NO_DECOR};
use crate::repr::{Decor, Decorate, Decorated, Formatted, SetSpan, Span, Spanned};
use crate::template::{HeredocTemplate, StringTemplate};
use crate::{Ident, InternalString, Number, RawString};
use std::fmt;
use std::ops::Range;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::expr::{BinaryOperator, UnaryOperator};

pub type Iter<'a> = Box<dyn Iterator<Item = &'a Expression> + 'a>;

pub type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Expression> + 'a>;

pub type ObjectIter<'a> = Box<dyn Iterator<Item = (&'a ObjectKey, &'a ObjectValue)> + 'a>;

pub type ObjectIterMut<'a> = Box<dyn Iterator<Item = (&'a ObjectKey, &'a mut ObjectValue)> + 'a>;

pub type TraversalIter<'a> = Box<dyn Iterator<Item = &'a Decorated<TraversalOperator>> + 'a>;

pub type TraversalIterMut<'a> = Box<dyn Iterator<Item = &'a mut Decorated<TraversalOperator>> + 'a>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Null(Decorated<Null>),
    Bool(Decorated<bool>),
    Number(Formatted<Number>),
    String(Decorated<InternalString>),
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

forward_decorate_span_impl!(Expression => {
    Null, Bool, Number, String, Array, Object, Template, HeredocTemplate, Parenthesis,
    Variable, ForExpr, Conditional, FuncCall, UnaryOp, BinaryOp, Traversal
});

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

impl From<Parenthesis> for Expression {
    fn from(value: Parenthesis) -> Self {
        Expression::Parenthesis(Box::new(value))
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode_decorated(&mut state, NO_DECOR)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

decorate_span_impl!(Parenthesis);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Array {
    values: Vec<Expression>,
    trailing: RawString,
    trailing_comma: bool,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Array);

impl Array {
    pub fn new(values: Vec<Expression>) -> Array {
        Array {
            values,
            trailing: RawString::default(),
            trailing_comma: false,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.values.iter())
    }

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
        self.trailing.despan(input);

        for value in &mut self.values {
            value.despan(input);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    items: Vec<(ObjectKey, ObjectValue)>,
    trailing: RawString,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Object);

impl Object {
    pub fn new(items: Vec<(ObjectKey, ObjectValue)>) -> Object {
        Object {
            items,
            trailing: RawString::default(),
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn iter(&self) -> ObjectIter<'_> {
        Box::new(self.items.iter().map(|(key, value)| (key, value)))
    }

    pub fn iter_mut(&mut self) -> ObjectIterMut<'_> {
        Box::new(self.items.iter_mut().map(|(key, value)| (&*key, value)))
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

        for (key, value) in &mut self.items {
            key.despan(input);
            value.despan(input);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKey {
    Ident(Decorated<Ident>),
    Expression(Expression),
}

forward_decorate_span_impl!(ObjectKey => { Ident, Expression });

impl ObjectKey {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            ObjectKey::Ident(ident) => ident.decor_mut().despan(input),
            ObjectKey::Expression(expr) => expr.despan(input),
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    cond_expr: Expression,
    true_expr: Expression,
    false_expr: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Conditional);

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

    pub fn cond_expr(&self) -> &Expression {
        &self.cond_expr
    }

    pub fn true_expr(&self) -> &Expression {
        &self.true_expr
    }

    pub fn false_expr(&self) -> &Expression {
        &self.false_expr
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.cond_expr.despan(input);
        self.true_expr.despan(input);
        self.false_expr.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncCall {
    name: Decorated<Ident>,
    args: FuncArgs,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(FuncCall);

impl FuncCall {
    pub fn new(name: Decorated<Ident>, args: FuncArgs) -> FuncCall {
        FuncCall {
            name,
            args,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn name(&self) -> &Decorated<Ident> {
        &self.name
    }

    pub fn args(&self) -> &FuncArgs {
        &self.args
    }

    pub fn args_mut(&mut self) -> &mut FuncArgs {
        &mut self.args
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.name.decor_mut().despan(input);
        self.args.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncArgs {
    args: Vec<Expression>,
    expand_final: bool,
    trailing: RawString,
    trailing_comma: bool,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(FuncArgs);

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

    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.args.iter())
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traversal {
    expr: Expression,
    operators: Vec<Decorated<TraversalOperator>>,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Traversal);

impl Traversal {
    pub fn new(expr: Expression, operators: Vec<Decorated<TraversalOperator>>) -> Traversal {
        Traversal {
            expr,
            operators,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    pub fn operators(&self) -> TraversalIter<'_> {
        Box::new(self.operators.iter())
    }

    pub fn operators_mut(&mut self) -> TraversalIterMut<'_> {
        Box::new(self.operators.iter_mut())
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);

        for operator in &mut self.operators {
            operator.despan(input);
        }
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

forward_decorate_span_impl!(TraversalOperator => { AttrSplat, FullSplat, GetAttr, Index, LegacyIndex });

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOp {
    operator: Spanned<UnaryOperator>,
    expr: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(UnaryOp);

impl UnaryOp {
    pub fn new(operator: Spanned<UnaryOperator>, expr: Expression) -> UnaryOp {
        UnaryOp {
            operator,
            expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn operator(&self) -> &Spanned<UnaryOperator> {
        &self.operator
    }

    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    lhs_expr: Expression,
    operator: Spanned<BinaryOperator>,
    rhs_expr: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(BinaryOp);

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

    pub fn lhs_expr(&self) -> &Expression {
        &self.lhs_expr
    }

    pub fn operator(&self) -> &Spanned<BinaryOperator> {
        &self.operator
    }

    pub fn rhs_expr(&self) -> &Expression {
        &self.rhs_expr
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.lhs_expr.despan(input);
        self.rhs_expr.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForExpr {
    intro: ForIntro,
    key_expr: Option<Expression>,
    value_expr: Expression,
    grouping: bool,
    cond: Option<ForCond>,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(ForExpr);

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

    pub fn intro(&self) -> &ForIntro {
        &self.intro
    }

    pub fn key_expr(&self) -> Option<&Expression> {
        self.key_expr.as_ref()
    }

    pub fn set_key_expr(&mut self, key_expr: Expression) {
        self.key_expr = Some(key_expr);
    }

    pub fn value_expr(&self) -> &Expression {
        &self.value_expr
    }

    pub fn grouping(&self) -> bool {
        self.grouping
    }

    pub fn set_grouping(&mut self, yes: bool) {
        self.grouping = yes;
    }

    pub fn cond(&self) -> Option<&ForCond> {
        self.cond.as_ref()
    }

    pub fn set_cond(&mut self, cond: ForCond) {
        self.cond = Some(cond);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForIntro {
    key_var: Option<Decorated<Ident>>,
    value_var: Decorated<Ident>,
    collection_expr: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(ForIntro);

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

    pub fn key_var(&self) -> Option<&Decorated<Ident>> {
        self.key_var.as_ref()
    }

    pub fn set_key_var(&mut self, key_var: Decorated<Ident>) {
        self.key_var = Some(key_var);
    }

    pub fn value_var(&self) -> &Decorated<Ident> {
        &self.value_var
    }

    pub fn collection_expr(&self) -> &Expression {
        &self.collection_expr
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForCond {
    expr: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(ForCond);

impl ForCond {
    pub fn new(expr: Expression) -> ForCond {
        ForCond {
            expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);
    }
}

impl From<Expression> for ForCond {
    fn from(value: Expression) -> Self {
        ForCond::new(value)
    }
}
