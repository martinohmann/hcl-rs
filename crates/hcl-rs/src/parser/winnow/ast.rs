#![allow(missing_docs)]

use super::encode::{Encode, EncodeDecorated, EncodeState, NO_DECOR};
use super::repr::{Decor, Decorate, Decorated, Despan, RawString, SetSpan, Span, Spanned};
use crate::expr::{self, BinaryOperator, HeredocStripMode, UnaryOperator, Variable};
use crate::structure;
use crate::template::{self, StripMode};
use crate::util::{dedent_by, min_leading_whitespace};
use crate::{Identifier, InternalString, Number};
use std::fmt;
use std::ops::Range;

macro_rules! forward_decorate_span_impl {
    ($ty:ident => { $($variant:ident),+ }) => {
        forward_decorate_impl!($ty => { $($variant),* });
        forward_span_impl!($ty => { $($variant),* });
    };
}

macro_rules! forward_decorate_impl {
    ($ty:ident => { $($variant:ident),+ }) => {
        impl Decorate for $ty {
            fn decor(&self) -> &Decor {
                match self {
                    $(
                        $ty::$variant(v) => v.decor(),
                    )*
                }
            }

            fn decor_mut(&mut self) -> &mut Decor {
                match self {
                    $(
                        $ty::$variant(v) => v.decor_mut(),
                    )*
                }
            }
        }
    };
}

macro_rules! forward_span_impl {
    ($ty:ident => { $($variant:ident),+ }) => {
        impl Span for $ty {
            fn span(&self) -> Option<Range<usize>> {
                match self {
                    $(
                        $ty::$variant(v) => v.span(),
                    )*
                }
            }
        }

        impl SetSpan for $ty {
            fn set_span(&mut self, span: Range<usize>) {
                match self {
                    $(
                        $ty::$variant(v) => v.set_span(span),
                    )*
                }
            }
        }
    };
}

macro_rules! decorate_span_impl {
    ($ty:ident) => {
        decorate_impl!($ty);
        span_impl!($ty);
    };
}

macro_rules! decorate_impl {
    ($ty:ident) => {
        impl Decorate for $ty {
            fn decor(&self) -> &Decor {
                &self.decor
            }

            fn decor_mut(&mut self) -> &mut Decor {
                &mut self.decor
            }
        }
    };
}

macro_rules! span_impl {
    ($ty:ident) => {
        impl Span for $ty {
            fn span(&self) -> Option<Range<usize>> {
                self.span.clone()
            }
        }

        impl SetSpan for $ty {
            fn set_span(&mut self, span: Range<usize>) {
                self.span = Some(span);
            }
        }
    };
}

pub type DecorOnly = Decorated<()>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Null(DecorOnly),
    Bool(Decorated<bool>),
    Number(Decorated<Number>),
    String(Decorated<InternalString>),
    Array(Box<Array>),
    Object(Box<Object>),
    Template(StringTemplate),
    HeredocTemplate(Box<HeredocTemplate>),
    Parenthesis(Box<Decorated<Expression>>),
    Variable(Decorated<Variable>),
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

impl Despan for Expression {
    fn despan(&mut self, input: &str) {
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

impl From<Expression> for expr::Expression {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::Null(_) => expr::Expression::Null,
            Expression::Bool(b) => expr::Expression::Bool(b.into_inner()),
            Expression::Number(n) => expr::Expression::Number(n.into_inner()),
            Expression::String(s) => expr::Expression::String(s.to_string()),
            Expression::Array(array) => expr::Expression::Array((*array).into()),
            Expression::Object(object) => expr::Expression::Object((*object).into()),
            Expression::Template(template) => {
                expr::TemplateExpr::QuotedString(template.into()).into()
            }
            Expression::HeredocTemplate(heredoc) => expr::Heredoc::from(*heredoc).into(),
            Expression::Parenthesis(expr) => {
                expr::Expression::Parenthesis(Box::new(expr.into_inner().into()))
            }
            Expression::Variable(var) => expr::Expression::Variable(var.into_inner()),
            Expression::ForExpr(expr) => expr::ForExpr::from(*expr).into(),
            Expression::Conditional(cond) => expr::Conditional::from(*cond).into(),
            Expression::FuncCall(call) => expr::FuncCall::from(*call).into(),
            Expression::UnaryOp(op) => expr::Operation::from(*op).into(),
            Expression::BinaryOp(op) => expr::Operation::from(*op).into(),
            Expression::Traversal(traversal) => expr::Traversal::from(*traversal).into(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f, None);
        self.encode_decorated(&mut state, NO_DECOR)
    }
}

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

    pub fn values(&self) -> &[Expression] {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut [Expression] {
        &mut self.values
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
}

impl Despan for Array {
    fn despan(&mut self, input: &str) {
        self.trailing.despan(input);

        for value in &mut self.values {
            value.despan(input);
        }
    }
}

impl From<Array> for Vec<expr::Expression> {
    fn from(array: Array) -> Self {
        array.values.into_iter().map(Into::into).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    items: Vec<ObjectItem>,
    trailing: RawString,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Object);

impl Object {
    pub fn new(items: Vec<ObjectItem>) -> Object {
        Object {
            items,
            trailing: RawString::default(),
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn items(&self) -> &[ObjectItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut [ObjectItem] {
        &mut self.items
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }
}

impl Despan for Object {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.trailing.despan(input);

        for item in &mut self.items {
            item.despan(input);
        }
    }
}

impl From<Object> for expr::Object<expr::ObjectKey, expr::Expression> {
    fn from(object: Object) -> Self {
        object
            .items
            .into_iter()
            .map(|item| (item.key.into(), item.value.into()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectItem {
    key: ObjectKey,
    key_value_separator: ObjectKeyValueSeparator,
    value: Expression,
    value_terminator: ObjectValueTerminator,
    span: Option<Range<usize>>,
}

span_impl!(ObjectItem);

impl ObjectItem {
    pub fn new(key: ObjectKey, value: Expression) -> ObjectItem {
        ObjectItem {
            key,
            key_value_separator: ObjectKeyValueSeparator::default(),
            value,
            value_terminator: ObjectValueTerminator::default(),
            span: None,
        }
    }

    pub fn key(&self) -> &ObjectKey {
        &self.key
    }

    pub fn key_mut(&mut self) -> &mut ObjectKey {
        &mut self.key
    }

    pub fn value(&self) -> &Expression {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Expression {
        &mut self.value
    }

    pub fn into_key(self) -> ObjectKey {
        self.key
    }

    pub fn into_value(self) -> Expression {
        self.value
    }

    pub fn into_key_value(self) -> (ObjectKey, Expression) {
        (self.key, self.value)
    }

    pub fn key_value_separator(&self) -> ObjectKeyValueSeparator {
        self.key_value_separator
    }

    pub fn value_terminator(&self) -> ObjectValueTerminator {
        self.value_terminator
    }

    pub fn set_key_value_separator(&mut self, sep: ObjectKeyValueSeparator) {
        self.key_value_separator = sep;
    }

    pub fn set_value_terminator(&mut self, terminator: ObjectValueTerminator) {
        self.value_terminator = terminator;
    }
}

impl Despan for ObjectItem {
    fn despan(&mut self, input: &str) {
        self.key.despan(input);
        self.value.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKey {
    Identifier(Decorated<Identifier>),
    Expression(Expression),
}

forward_decorate_span_impl!(ObjectKey => { Identifier, Expression });

impl Despan for ObjectKey {
    fn despan(&mut self, input: &str) {
        match self {
            ObjectKey::Identifier(ident) => ident.decor_mut().despan(input),
            ObjectKey::Expression(expr) => expr.despan(input),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKeyValueSeparator {
    Colon,
    Equals,
}

impl Default for ObjectKeyValueSeparator {
    fn default() -> Self {
        ObjectKeyValueSeparator::Equals
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectValueTerminator {
    None,
    Newline,
    Comma,
}

impl Default for ObjectValueTerminator {
    fn default() -> Self {
        ObjectValueTerminator::Comma
    }
}

impl From<ObjectKey> for expr::ObjectKey {
    fn from(key: ObjectKey) -> Self {
        match key {
            ObjectKey::Identifier(ident) => expr::ObjectKey::Identifier(ident.into_inner()),
            ObjectKey::Expression(expr) => expr::ObjectKey::Expression(expr.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeredocTemplate {
    delimiter: Identifier,
    template: Template,
    indent: Option<usize>,
    trailing: RawString,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(HeredocTemplate);

impl HeredocTemplate {
    pub fn new(delimiter: Identifier, template: Template) -> HeredocTemplate {
        HeredocTemplate {
            delimiter,
            template,
            indent: None,
            trailing: RawString::default(),
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn delimiter(&self) -> &Identifier {
        &self.delimiter
    }

    pub fn template(&self) -> &Template {
        &self.template
    }

    pub fn template_mut(&mut self) -> &mut Template {
        &mut self.template
    }

    pub fn indent(&self) -> Option<usize> {
        self.indent
    }

    pub fn set_indent(&mut self, indent: usize) {
        self.indent = Some(indent);
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    pub fn dedent(&mut self) {
        let mut indent = usize::MAX;
        let mut skip_first = false;

        for element in &self.template.elements {
            match element {
                Element::Literal(literal) => {
                    let leading_ws = min_leading_whitespace(literal, skip_first);
                    indent = indent.min(leading_ws);
                    skip_first = literal.ends_with('\n');
                }
                _other => skip_first = true,
            }
        }

        skip_first = false;

        for element in &mut self.template.elements {
            match element {
                Element::Literal(literal) => {
                    let dedented = dedent_by(literal, indent, skip_first);
                    *literal.as_mut() = dedented.into();
                    skip_first = literal.ends_with('\n');
                }
                _other => skip_first = true,
            }
        }

        self.set_indent(indent);
    }
}

impl Despan for HeredocTemplate {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.template.despan(input);
        self.trailing.despan(input);
    }
}

impl From<HeredocTemplate> for expr::Heredoc {
    fn from(heredoc: HeredocTemplate) -> Self {
        let strip = heredoc
            .indent
            .map_or(HeredocStripMode::None, |_| HeredocStripMode::Indent);

        expr::Heredoc {
            delimiter: heredoc.delimiter,
            template: heredoc.template.into(),
            strip,
        }
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
}

impl Despan for Conditional {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.cond_expr.despan(input);
        self.true_expr.despan(input);
        self.false_expr.despan(input);
    }
}

impl From<Conditional> for expr::Conditional {
    fn from(cond: Conditional) -> Self {
        expr::Conditional {
            cond_expr: cond.cond_expr.into(),
            true_expr: cond.true_expr.into(),
            false_expr: cond.false_expr.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncCall {
    name: Decorated<Identifier>,
    signature: FuncSig,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(FuncCall);

impl FuncCall {
    pub fn new(name: Decorated<Identifier>, signature: FuncSig) -> FuncCall {
        FuncCall {
            name,
            signature,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn name(&self) -> &Decorated<Identifier> {
        &self.name
    }

    pub fn signature(&self) -> &FuncSig {
        &self.signature
    }
}

impl Despan for FuncCall {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.name.decor_mut().despan(input);
        self.signature.despan(input);
    }
}

impl From<FuncCall> for expr::FuncCall {
    fn from(call: FuncCall) -> Self {
        expr::FuncCall {
            name: call.name.into_inner(),
            args: call.signature.args.into_iter().map(Into::into).collect(),
            expand_final: call.signature.expand_final,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncSig {
    args: Vec<Expression>,
    expand_final: bool,
    trailing: RawString,
    trailing_comma: bool,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(FuncSig);

impl FuncSig {
    pub fn new(args: Vec<Expression>) -> FuncSig {
        FuncSig {
            args,
            expand_final: false,
            trailing: RawString::default(),
            trailing_comma: false,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn args(&self) -> &[Expression] {
        &self.args
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
}

impl Despan for FuncSig {
    fn despan(&mut self, input: &str) {
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

    pub fn operators(&self) -> &[Decorated<TraversalOperator>] {
        &self.operators
    }
}

impl Despan for Traversal {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);

        for operator in &mut self.operators {
            operator.despan(input);
        }
    }
}

impl From<Traversal> for expr::Traversal {
    fn from(traversal: Traversal) -> Self {
        expr::Traversal {
            expr: traversal.expr.into(),
            operators: traversal
                .operators
                .into_iter()
                .map(Decorated::into_inner)
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalOperator {
    AttrSplat(DecorOnly),
    FullSplat(DecorOnly),
    GetAttr(Decorated<Identifier>),
    Index(Expression),
    LegacyIndex(Decorated<u64>),
}

forward_decorate_span_impl!(TraversalOperator => { AttrSplat, FullSplat, GetAttr, Index, LegacyIndex });

impl Despan for TraversalOperator {
    fn despan(&mut self, input: &str) {
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

impl From<TraversalOperator> for expr::TraversalOperator {
    fn from(operator: TraversalOperator) -> Self {
        match operator {
            TraversalOperator::AttrSplat(_) => expr::TraversalOperator::AttrSplat,
            TraversalOperator::FullSplat(_) => expr::TraversalOperator::FullSplat,
            TraversalOperator::GetAttr(ident) => {
                expr::TraversalOperator::GetAttr(ident.into_inner())
            }
            TraversalOperator::Index(expr) => expr::TraversalOperator::Index(expr.into()),
            TraversalOperator::LegacyIndex(index) => {
                expr::TraversalOperator::LegacyIndex(index.into_inner())
            }
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

    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    pub fn operator(&self) -> &Spanned<UnaryOperator> {
        &self.operator
    }
}

impl Despan for UnaryOp {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);
    }
}

impl From<UnaryOp> for expr::UnaryOp {
    fn from(op: UnaryOp) -> Self {
        expr::UnaryOp {
            operator: op.operator.into_inner(),
            expr: op.expr.into(),
        }
    }
}

impl From<UnaryOp> for expr::Operation {
    fn from(op: UnaryOp) -> Self {
        expr::Operation::Unary(op.into())
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

    pub fn rhs_expr(&self) -> &Expression {
        &self.rhs_expr
    }

    pub fn operator(&self) -> &Spanned<BinaryOperator> {
        &self.operator
    }
}

impl Despan for BinaryOp {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.lhs_expr.despan(input);
        self.rhs_expr.despan(input);
    }
}

impl From<BinaryOp> for expr::BinaryOp {
    fn from(op: BinaryOp) -> Self {
        expr::BinaryOp {
            lhs_expr: op.lhs_expr.into(),
            operator: op.operator.into_inner(),
            rhs_expr: op.rhs_expr.into(),
        }
    }
}

impl From<BinaryOp> for expr::Operation {
    fn from(op: BinaryOp) -> Self {
        expr::Operation::Binary(op.into())
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
}

impl Despan for ForExpr {
    fn despan(&mut self, input: &str) {
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

impl From<ForExpr> for expr::ForExpr {
    fn from(expr: ForExpr) -> Self {
        expr::ForExpr {
            key_var: expr.intro.key_var.map(Decorated::into_inner),
            value_var: expr.intro.value_var.into_inner(),
            collection_expr: expr.intro.collection_expr.into(),
            key_expr: expr.key_expr.map(Into::into),
            value_expr: expr.value_expr.into(),
            grouping: expr.grouping,
            cond_expr: expr.cond.map(|cond| cond.expr.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForIntro {
    key_var: Option<Decorated<Identifier>>,
    value_var: Decorated<Identifier>,
    collection_expr: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(ForIntro);

impl ForIntro {
    pub fn new(value_var: Decorated<Identifier>, collection_expr: Expression) -> ForIntro {
        ForIntro {
            key_var: None,
            value_var,
            collection_expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn key_var(&self) -> Option<&Decorated<Identifier>> {
        self.key_var.as_ref()
    }

    pub fn set_key_var(&mut self, key_var: Decorated<Identifier>) {
        self.key_var = Some(key_var);
    }

    pub fn value_var(&self) -> &Decorated<Identifier> {
        &self.value_var
    }

    pub fn collection_expr(&self) -> &Expression {
        &self.collection_expr
    }
}

impl Despan for ForIntro {
    fn despan(&mut self, input: &str) {
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
}

impl Despan for ForCond {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);
    }
}

impl From<Expression> for ForCond {
    fn from(value: Expression) -> Self {
        ForCond::new(value)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Body {
    structures: Vec<Structure>,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Body);

impl Body {
    pub fn new(structures: Vec<Structure>) -> Body {
        Body {
            structures,
            ..Default::default()
        }
    }

    pub fn structures(&self) -> &[Structure] {
        &self.structures
    }
}

impl Despan for Body {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        for structure in &mut self.structures {
            structure.despan(input);
        }
    }
}

impl From<Vec<Structure>> for Body {
    fn from(structures: Vec<Structure>) -> Self {
        Body::new(structures)
    }
}

impl From<Body> for structure::Body {
    fn from(body: Body) -> Self {
        structure::Body::from_iter(body.structures)
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f, None);
        self.encode_decorated(&mut state, NO_DECOR)
    }
}

#[derive(Debug, Clone)]
pub enum Structure {
    Attribute(Box<Attribute>),
    Block(Box<Block>),
}

forward_decorate_span_impl!(Structure => { Attribute, Block });

impl Despan for Structure {
    fn despan(&mut self, input: &str) {
        match self {
            Structure::Attribute(attr) => attr.despan(input),
            Structure::Block(block) => block.despan(input),
        }
    }
}

impl From<Structure> for structure::Structure {
    fn from(structure: Structure) -> Self {
        match structure {
            Structure::Attribute(attr) => structure::Structure::Attribute((*attr).into()),
            Structure::Block(block) => structure::Structure::Block((*block).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    key: Decorated<Identifier>,
    expr: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Attribute);

impl Attribute {
    pub fn new(key: Decorated<Identifier>, expr: Expression) -> Attribute {
        Attribute {
            key,
            expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn key(&self) -> &Decorated<Identifier> {
        &self.key
    }

    pub fn expr(&self) -> &Expression {
        &self.expr
    }
}

impl Despan for Attribute {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.key.decor_mut().despan(input);
        self.expr.despan(input);
    }
}

impl From<Attribute> for structure::Attribute {
    fn from(attr: Attribute) -> Self {
        structure::Attribute {
            key: attr.key.into_inner(),
            expr: attr.expr.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    identifier: Decorated<Identifier>,
    labels: Vec<BlockLabel>,
    body: BlockBody,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Block);

impl Block {
    pub fn new(ident: Decorated<Identifier>, body: BlockBody) -> Block {
        Block::new_with_labels(ident, Vec::new(), body)
    }

    pub fn new_with_labels(
        ident: Decorated<Identifier>,
        labels: Vec<BlockLabel>,
        body: BlockBody,
    ) -> Block {
        Block {
            identifier: ident,
            labels,
            body,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn ident(&self) -> &Decorated<Identifier> {
        &self.identifier
    }

    pub fn labels(&self) -> &[BlockLabel] {
        &self.labels
    }

    pub fn body(&self) -> &BlockBody {
        &self.body
    }
}

impl Despan for Block {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.identifier.decor_mut().despan(input);
        for label in &mut self.labels {
            label.despan(input);
        }
        self.body.despan(input);
    }
}

impl From<Block> for structure::Block {
    fn from(block: Block) -> Self {
        structure::Block {
            identifier: block.identifier.into_inner(),
            labels: block.labels.into_iter().map(Into::into).collect(),
            body: block.body.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockLabel {
    Identifier(Decorated<Identifier>),
    String(Decorated<InternalString>),
}

forward_decorate_span_impl!(BlockLabel => { Identifier, String });

impl Despan for BlockLabel {
    fn despan(&mut self, input: &str) {
        match self {
            BlockLabel::Identifier(ident) => ident.decor_mut().despan(input),
            BlockLabel::String(expr) => expr.decor_mut().despan(input),
        }
    }
}

impl From<BlockLabel> for structure::BlockLabel {
    fn from(label: BlockLabel) -> Self {
        match label {
            BlockLabel::Identifier(ident) => structure::BlockLabel::Identifier(ident.into_inner()),
            BlockLabel::String(expr) => structure::BlockLabel::String(expr.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockBody {
    Multiline(Box<Body>),
    Oneline(Box<Attribute>),
    Empty(RawString),
}

impl Despan for BlockBody {
    fn despan(&mut self, input: &str) {
        match self {
            BlockBody::Multiline(body) => body.despan(input),
            BlockBody::Oneline(attr) => attr.despan(input),
            BlockBody::Empty(raw) => raw.despan(input),
        }
    }
}

impl From<BlockBody> for structure::Body {
    fn from(body: BlockBody) -> Self {
        match body {
            BlockBody::Multiline(body) => (*body).into(),
            BlockBody::Oneline(attr) => structure::Attribute::from(*attr).into(),
            BlockBody::Empty(_) => structure::Body::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StringTemplate {
    elements: Vec<Element>,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(StringTemplate);

impl StringTemplate {
    pub fn new(elements: Vec<Element>) -> StringTemplate {
        StringTemplate {
            elements,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn elements_mut(&mut self) -> &mut [Element] {
        &mut self.elements
    }
}

impl Despan for StringTemplate {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        for element in &mut self.elements {
            element.despan(input);
        }
    }
}

impl From<StringTemplate> for template::Template {
    fn from(template: StringTemplate) -> Self {
        template
            .elements
            .into_iter()
            .map(template::Element::from)
            .collect()
    }
}

impl From<StringTemplate> for String {
    fn from(template: StringTemplate) -> Self {
        template::Template::from(template).to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template {
    elements: Vec<Element>,
    span: Option<Range<usize>>,
}

span_impl!(Template);

impl Template {
    pub fn new(elements: Vec<Element>) -> Template {
        Template {
            elements,
            span: None,
        }
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn elements_mut(&mut self) -> &mut [Element] {
        &mut self.elements
    }
}

impl Despan for Template {
    fn despan(&mut self, input: &str) {
        for element in &mut self.elements {
            element.despan(input);
        }
    }
}

impl From<Template> for template::Template {
    fn from(template: Template) -> Self {
        template
            .elements
            .into_iter()
            .map(template::Element::from)
            .collect()
    }
}

impl From<Template> for String {
    fn from(template: Template) -> Self {
        template::Template::from(template).to_string()
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f, None);
        self.encode(&mut state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    Literal(Spanned<InternalString>),
    Interpolation(Interpolation),
    Directive(Directive),
}

forward_span_impl!(Element => { Literal, Interpolation, Directive });

impl Despan for Element {
    fn despan(&mut self, input: &str) {
        match self {
            Element::Literal(_) => {}
            Element::Interpolation(interp) => interp.despan(input),
            Element::Directive(dir) => dir.despan(input),
        }
    }
}

impl From<Element> for template::Element {
    fn from(element: Element) -> Self {
        match element {
            Element::Literal(lit) => template::Element::Literal(lit.to_string()),
            Element::Interpolation(interp) => template::Element::Interpolation(interp.into()),
            Element::Directive(dir) => template::Element::Directive(dir.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpolation {
    expr: Expression,
    strip: StripMode,
    span: Option<Range<usize>>,
}

span_impl!(Interpolation);

impl Interpolation {
    pub fn new(expr: Expression, strip: StripMode) -> Interpolation {
        Interpolation {
            expr,
            strip,
            span: None,
        }
    }

    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    pub fn strip(&self) -> StripMode {
        self.strip
    }
}

impl Despan for Interpolation {
    fn despan(&mut self, input: &str) {
        self.expr.despan(input);
    }
}

impl From<Interpolation> for template::Interpolation {
    fn from(interp: Interpolation) -> Self {
        template::Interpolation {
            expr: interp.expr.into(),
            strip: interp.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    If(IfDirective),
    For(ForDirective),
}

forward_span_impl!(Directive => { If, For });

impl Despan for Directive {
    fn despan(&mut self, input: &str) {
        match self {
            Directive::If(dir) => dir.despan(input),
            Directive::For(dir) => dir.despan(input),
        }
    }
}

impl From<Directive> for template::Directive {
    fn from(dir: Directive) -> Self {
        match dir {
            Directive::If(dir) => template::Directive::If(dir.into()),
            Directive::For(dir) => template::Directive::For(dir.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfDirective {
    if_expr: IfTemplateExpr,
    else_expr: Option<ElseTemplateExpr>,
    endif_expr: EndifTemplateExpr,
    span: Option<Range<usize>>,
}

span_impl!(IfDirective);

impl IfDirective {
    pub fn new(
        if_expr: IfTemplateExpr,
        else_expr: Option<ElseTemplateExpr>,
        endif_expr: EndifTemplateExpr,
    ) -> IfDirective {
        IfDirective {
            if_expr,
            else_expr,
            endif_expr,
            span: None,
        }
    }

    pub fn if_expr(&self) -> &IfTemplateExpr {
        &self.if_expr
    }

    pub fn else_expr(&self) -> Option<&ElseTemplateExpr> {
        self.else_expr.as_ref()
    }

    pub fn endif_expr(&self) -> &EndifTemplateExpr {
        &self.endif_expr
    }
}

impl Despan for IfDirective {
    fn despan(&mut self, input: &str) {
        self.if_expr.despan(input);

        if let Some(else_expr) = &mut self.else_expr {
            else_expr.despan(input);
        }

        self.endif_expr.despan(input);
    }
}

impl From<IfDirective> for template::IfDirective {
    fn from(dir: IfDirective) -> Self {
        let else_strip = dir
            .else_expr
            .as_ref()
            .map(|expr| expr.strip)
            .unwrap_or_default();

        template::IfDirective {
            cond_expr: dir.if_expr.cond_expr.into(),
            true_template: dir.if_expr.template.into(),
            false_template: dir.else_expr.map(|expr| expr.template.into()),
            if_strip: dir.if_expr.strip,
            else_strip,
            endif_strip: dir.endif_expr.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfTemplateExpr {
    preamble: RawString,
    cond_expr: Expression,
    template: Template,
    strip: StripMode,
}

impl IfTemplateExpr {
    pub fn new(cond_expr: Expression, template: Template, strip: StripMode) -> IfTemplateExpr {
        IfTemplateExpr {
            preamble: RawString::default(),
            cond_expr,
            template,
            strip,
        }
    }

    pub fn cond_expr(&self) -> &Expression {
        &self.cond_expr
    }

    pub fn template(&self) -> &Template {
        &self.template
    }

    pub fn strip(&self) -> StripMode {
        self.strip
    }

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }
}

impl Despan for IfTemplateExpr {
    fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.cond_expr.despan(input);
        self.template.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElseTemplateExpr {
    preamble: RawString,
    trailing: RawString,
    template: Template,
    strip: StripMode,
}

impl ElseTemplateExpr {
    pub fn new(template: Template, strip: StripMode) -> ElseTemplateExpr {
        ElseTemplateExpr {
            preamble: RawString::default(),
            trailing: RawString::default(),
            template,
            strip,
        }
    }

    pub fn template(&self) -> &Template {
        &self.template
    }

    pub fn strip(&self) -> StripMode {
        self.strip
    }

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }
}

impl Despan for ElseTemplateExpr {
    fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.template.despan(input);
        self.trailing.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndifTemplateExpr {
    preamble: RawString,
    trailing: RawString,
    strip: StripMode,
}

impl EndifTemplateExpr {
    pub fn new(strip: StripMode) -> EndifTemplateExpr {
        EndifTemplateExpr {
            preamble: RawString::default(),
            trailing: RawString::default(),
            strip,
        }
    }

    pub fn strip(&self) -> StripMode {
        self.strip
    }

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }
}

impl Despan for EndifTemplateExpr {
    fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.trailing.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForDirective {
    for_expr: ForTemplateExpr,
    endfor_expr: EndforTemplateExpr,
    span: Option<Range<usize>>,
}

span_impl!(ForDirective);

impl ForDirective {
    pub fn new(for_expr: ForTemplateExpr, endfor_expr: EndforTemplateExpr) -> ForDirective {
        ForDirective {
            for_expr,
            endfor_expr,
            span: None,
        }
    }

    pub fn for_expr(&self) -> &ForTemplateExpr {
        &self.for_expr
    }

    pub fn endfor_expr(&self) -> &EndforTemplateExpr {
        &self.endfor_expr
    }
}

impl Despan for ForDirective {
    fn despan(&mut self, input: &str) {
        self.for_expr.despan(input);
        self.endfor_expr.despan(input);
    }
}

impl From<ForDirective> for template::ForDirective {
    fn from(dir: ForDirective) -> Self {
        template::ForDirective {
            key_var: dir.for_expr.key_var.map(Decorated::into_inner),
            value_var: dir.for_expr.value_var.into_inner(),
            collection_expr: dir.for_expr.collection_expr.into(),
            template: dir.for_expr.template.into(),
            for_strip: dir.for_expr.strip,
            endfor_strip: dir.endfor_expr.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForTemplateExpr {
    preamble: RawString,
    key_var: Option<Decorated<Identifier>>,
    value_var: Decorated<Identifier>,
    collection_expr: Expression,
    template: Template,
    strip: StripMode,
}

impl ForTemplateExpr {
    pub fn new(
        key_var: Option<Decorated<Identifier>>,
        value_var: Decorated<Identifier>,
        collection_expr: Expression,
        template: Template,
        strip: StripMode,
    ) -> ForTemplateExpr {
        ForTemplateExpr {
            preamble: RawString::default(),
            key_var,
            value_var,
            collection_expr,
            template,
            strip,
        }
    }

    pub fn key_var(&self) -> Option<&Decorated<Identifier>> {
        self.key_var.as_ref()
    }

    pub fn value_var(&self) -> &Decorated<Identifier> {
        &self.value_var
    }

    pub fn collection_expr(&self) -> &Expression {
        &self.collection_expr
    }

    pub fn template(&self) -> &Template {
        &self.template
    }

    pub fn strip(&self) -> StripMode {
        self.strip
    }

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }
}

impl Despan for ForTemplateExpr {
    fn despan(&mut self, input: &str) {
        self.preamble.despan(input);

        if let Some(key_var) = &mut self.key_var {
            key_var.decor_mut().despan(input);
        }

        self.value_var.decor_mut().despan(input);
        self.collection_expr.despan(input);
        self.template.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndforTemplateExpr {
    preamble: RawString,
    trailing: RawString,
    strip: StripMode,
}

impl EndforTemplateExpr {
    pub fn new(strip: StripMode) -> EndforTemplateExpr {
        EndforTemplateExpr {
            preamble: RawString::default(),
            trailing: RawString::default(),
            strip,
        }
    }

    pub fn strip(&self) -> StripMode {
        self.strip
    }

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }
}

impl Despan for EndforTemplateExpr {
    fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.trailing.despan(input);
    }
}
