#![allow(missing_docs)]

use super::encode::{Encode, EncodeDecorated, EncodeState, NO_DECOR};
use super::repr::{Decor, Decorate, Decorated, Despan, RawString, Span, Spanned};
use crate::expr::{self, BinaryOperator, HeredocStripMode, UnaryOperator, Variable};
use crate::structure;
use crate::template::{self, StripMode};
use crate::{Identifier, InternalString, Number};
use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Null(Decorated<()>),
    Bool(Decorated<bool>),
    Number(Decorated<Number>),
    String(Decorated<InternalString>),
    Array(Box<Decorated<Array>>),
    Object(Box<Decorated<Object>>),
    Template(Decorated<Template>),
    HeredocTemplate(Box<Decorated<HeredocTemplate>>),
    Parenthesis(Box<Decorated<Expression>>),
    Variable(Decorated<Variable>),
    Conditional(Box<Decorated<Conditional>>),
    FuncCall(Box<Decorated<FuncCall>>),
    Traversal(Box<Decorated<Traversal>>),
    UnaryOp(Box<Decorated<UnaryOp>>),
    BinaryOp(Box<Decorated<BinaryOp>>),
    ForExpr(Box<Decorated<ForExpr>>),
}

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

impl Decorate for Expression {
    fn decor(&self) -> &Decor {
        match self {
            Expression::Null(n) => n.decor(),
            Expression::Bool(b) => b.decor(),
            Expression::Number(n) => n.decor(),
            Expression::String(s) => s.decor(),
            Expression::Array(array) => array.decor(),
            Expression::Object(object) => object.decor(),
            Expression::Template(template) => template.decor(),
            Expression::HeredocTemplate(heredoc) => heredoc.decor(),
            Expression::Parenthesis(expr) => expr.decor(),
            Expression::Variable(var) => var.decor(),
            Expression::ForExpr(expr) => expr.decor(),
            Expression::Conditional(cond) => cond.decor(),
            Expression::FuncCall(call) => call.decor(),
            Expression::UnaryOp(op) => op.decor(),
            Expression::BinaryOp(op) => op.decor(),
            Expression::Traversal(traversal) => traversal.decor(),
        }
    }

    fn decor_mut(&mut self) -> &mut Decor {
        match self {
            Expression::Null(n) => n.decor_mut(),
            Expression::Bool(b) => b.decor_mut(),
            Expression::Number(n) => n.decor_mut(),
            Expression::String(s) => s.decor_mut(),
            Expression::Array(array) => array.decor_mut(),
            Expression::Object(object) => object.decor_mut(),
            Expression::Template(template) => template.decor_mut(),
            Expression::HeredocTemplate(heredoc) => heredoc.decor_mut(),
            Expression::Parenthesis(expr) => expr.decor_mut(),
            Expression::Variable(var) => var.decor_mut(),
            Expression::ForExpr(expr) => expr.decor_mut(),
            Expression::Conditional(cond) => cond.decor_mut(),
            Expression::FuncCall(call) => call.decor_mut(),
            Expression::UnaryOp(op) => op.decor_mut(),
            Expression::BinaryOp(op) => op.decor_mut(),
            Expression::Traversal(traversal) => traversal.decor_mut(),
        }
    }
}

impl Span for Expression {
    fn span(&self) -> Option<Range<usize>> {
        match self {
            Expression::Null(n) => n.span(),
            Expression::Bool(b) => b.span(),
            Expression::Number(n) => n.span(),
            Expression::String(s) => s.span(),
            Expression::Array(array) => array.span(),
            Expression::Object(object) => object.span(),
            Expression::Template(template) => template.span(),
            Expression::HeredocTemplate(heredoc) => heredoc.span(),
            Expression::Parenthesis(expr) => expr.span(),
            Expression::Variable(var) => var.span(),
            Expression::ForExpr(expr) => expr.span(),
            Expression::Conditional(cond) => cond.span(),
            Expression::FuncCall(call) => call.span(),
            Expression::UnaryOp(op) => op.span(),
            Expression::BinaryOp(op) => op.span(),
            Expression::Traversal(traversal) => traversal.span(),
        }
    }

    fn set_span(&mut self, span: Range<usize>) {
        match self {
            Expression::Null(n) => n.set_span(span),
            Expression::Bool(b) => b.set_span(span),
            Expression::Number(n) => n.set_span(span),
            Expression::String(s) => s.set_span(span),
            Expression::Array(array) => array.set_span(span),
            Expression::Object(object) => object.set_span(span),
            Expression::Template(template) => template.set_span(span),
            Expression::HeredocTemplate(heredoc) => heredoc.set_span(span),
            Expression::Parenthesis(expr) => expr.set_span(span),
            Expression::Variable(var) => var.set_span(span),
            Expression::ForExpr(expr) => expr.set_span(span),
            Expression::Conditional(cond) => cond.set_span(span),
            Expression::FuncCall(call) => call.set_span(span),
            Expression::UnaryOp(op) => op.set_span(span),
            Expression::BinaryOp(op) => op.set_span(span),
            Expression::Traversal(traversal) => traversal.set_span(span),
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
            Expression::Array(array) => expr::Expression::Array(array.inner_into()),
            Expression::Object(object) => expr::Expression::Object(object.inner_into()),
            Expression::Template(template) => {
                expr::TemplateExpr::QuotedString(template.inner_into()).into()
            }
            Expression::HeredocTemplate(heredoc) => {
                expr::Heredoc::from(heredoc.into_inner()).into()
            }
            Expression::Parenthesis(expr) => {
                expr::Expression::Parenthesis(Box::new(expr.inner_into()))
            }
            Expression::Variable(var) => expr::Expression::Variable(var.into_inner()),
            Expression::ForExpr(expr) => expr::ForExpr::from(expr.into_inner()).into(),
            Expression::Conditional(cond) => expr::Conditional::from(cond.into_inner()).into(),
            Expression::FuncCall(call) => expr::FuncCall::from(call.into_inner()).into(),
            Expression::UnaryOp(op) => expr::Operation::from(op.into_inner()).into(),
            Expression::BinaryOp(op) => expr::Operation::from(op.into_inner()).into(),
            Expression::Traversal(traversal) => {
                expr::Traversal::from(traversal.into_inner()).into()
            }
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
}

impl Array {
    pub fn new(values: Vec<Expression>) -> Array {
        Array {
            values,
            trailing: RawString::default(),
            trailing_comma: false,
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
    items: Vec<Spanned<ObjectItem>>,
    trailing: RawString,
}

impl Object {
    pub fn new(items: Vec<Spanned<ObjectItem>>) -> Object {
        Object {
            items,
            trailing: RawString::default(),
        }
    }

    pub fn items(&self) -> &[Spanned<ObjectItem>] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut [Spanned<ObjectItem>] {
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
            .map(Spanned::into_inner)
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
}

impl ObjectItem {
    pub fn new(key: ObjectKey, value: Expression) -> ObjectItem {
        ObjectItem {
            key,
            key_value_separator: ObjectKeyValueSeparator::default(),
            value,
            value_terminator: ObjectValueTerminator::default(),
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

impl Despan for ObjectKey {
    fn despan(&mut self, input: &str) {
        match self {
            ObjectKey::Identifier(ident) => ident.decor_mut().despan(input),
            ObjectKey::Expression(expr) => expr.despan(input),
        }
    }
}

impl Decorate for ObjectKey {
    fn decor(&self) -> &Decor {
        match self {
            ObjectKey::Identifier(ident) => ident.decor(),
            ObjectKey::Expression(expr) => expr.decor(),
        }
    }

    fn decor_mut(&mut self) -> &mut Decor {
        match self {
            ObjectKey::Identifier(ident) => ident.decor_mut(),
            ObjectKey::Expression(expr) => expr.decor_mut(),
        }
    }
}

impl Span for ObjectKey {
    fn span(&self) -> Option<Range<usize>> {
        match self {
            ObjectKey::Identifier(ident) => ident.span(),
            ObjectKey::Expression(expr) => expr.span(),
        }
    }

    fn set_span(&mut self, span: Range<usize>) {
        match self {
            ObjectKey::Identifier(ident) => ident.set_span(span),
            ObjectKey::Expression(expr) => expr.set_span(span),
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
    delimiter: Decorated<Identifier>,
    template: Spanned<Template>,
    strip: HeredocStripMode,
    trailing: RawString,
}

impl HeredocTemplate {
    pub fn new(
        delimiter: Decorated<Identifier>,
        strip: HeredocStripMode,
        template: Spanned<Template>,
    ) -> HeredocTemplate {
        HeredocTemplate {
            delimiter,
            template,
            strip,
            trailing: RawString::default(),
        }
    }

    pub fn delimiter(&self) -> &Decorated<Identifier> {
        &self.delimiter
    }

    pub fn template(&self) -> &Spanned<Template> {
        &self.template
    }

    pub fn strip(&self) -> HeredocStripMode {
        self.strip
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }
}

impl Despan for HeredocTemplate {
    fn despan(&mut self, input: &str) {
        self.delimiter.decor_mut().despan(input);
        self.template.despan(input);
    }
}

impl From<HeredocTemplate> for expr::Heredoc {
    fn from(heredoc: HeredocTemplate) -> Self {
        expr::Heredoc {
            delimiter: heredoc.delimiter.into_inner(),
            template: heredoc.template.inner_into(),
            strip: heredoc.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    cond_expr: Expression,
    true_expr: Expression,
    false_expr: Expression,
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
    signature: Decorated<FuncSig>,
}

impl FuncCall {
    pub fn new(name: Decorated<Identifier>, signature: Decorated<FuncSig>) -> FuncCall {
        FuncCall { name, signature }
    }

    pub fn name(&self) -> &Decorated<Identifier> {
        &self.name
    }

    pub fn signature(&self) -> &Decorated<FuncSig> {
        &self.signature
    }
}

impl Despan for FuncCall {
    fn despan(&mut self, input: &str) {
        self.name.decor_mut().despan(input);
        self.signature.despan(input);
    }
}

impl From<FuncCall> for expr::FuncCall {
    fn from(call: FuncCall) -> Self {
        let signature = call.signature.into_inner();

        expr::FuncCall {
            name: call.name.into_inner(),
            args: signature.args.into_iter().map(Into::into).collect(),
            expand_final: signature.expand_final,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncSig {
    args: Vec<Expression>,
    expand_final: bool,
    trailing: RawString,
    trailing_comma: bool,
}

impl FuncSig {
    pub fn new(args: Vec<Expression>) -> FuncSig {
        FuncSig {
            args,
            expand_final: false,
            trailing: RawString::default(),
            trailing_comma: false,
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
}

impl Traversal {
    pub fn new(expr: Expression, operators: Vec<Decorated<TraversalOperator>>) -> Traversal {
        Traversal { expr, operators }
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
                .map(Decorated::inner_into)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalOperator {
    AttrSplat(Decorated<()>),
    FullSplat(Decorated<()>),
    GetAttr(Decorated<Identifier>),
    Index(Expression),
    LegacyIndex(Decorated<u64>),
}

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

impl Decorate for TraversalOperator {
    fn decor(&self) -> &Decor {
        match self {
            TraversalOperator::AttrSplat(splat) | TraversalOperator::FullSplat(splat) => {
                splat.decor()
            }
            TraversalOperator::GetAttr(ident) => ident.decor(),
            TraversalOperator::Index(expr) => expr.decor(),
            TraversalOperator::LegacyIndex(index) => index.decor(),
        }
    }

    fn decor_mut(&mut self) -> &mut Decor {
        match self {
            TraversalOperator::AttrSplat(splat) | TraversalOperator::FullSplat(splat) => {
                splat.decor_mut()
            }
            TraversalOperator::GetAttr(ident) => ident.decor_mut(),
            TraversalOperator::Index(expr) => expr.decor_mut(),
            TraversalOperator::LegacyIndex(index) => index.decor_mut(),
        }
    }
}

impl Span for TraversalOperator {
    fn span(&self) -> Option<Range<usize>> {
        match self {
            TraversalOperator::AttrSplat(splat) | TraversalOperator::FullSplat(splat) => {
                splat.span()
            }
            TraversalOperator::GetAttr(ident) => ident.span(),
            TraversalOperator::Index(expr) => expr.span(),
            TraversalOperator::LegacyIndex(index) => index.span(),
        }
    }

    fn set_span(&mut self, span: Range<usize>) {
        match self {
            TraversalOperator::AttrSplat(splat) | TraversalOperator::FullSplat(splat) => {
                splat.set_span(span);
            }
            TraversalOperator::GetAttr(ident) => ident.set_span(span),
            TraversalOperator::Index(expr) => expr.set_span(span),
            TraversalOperator::LegacyIndex(index) => index.set_span(span),
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
}

impl UnaryOp {
    pub fn new(operator: Spanned<UnaryOperator>, expr: Expression) -> UnaryOp {
        UnaryOp { operator, expr }
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
    operator: Decorated<BinaryOperator>,
    rhs_expr: Expression,
}

impl BinaryOp {
    pub fn new(
        lhs_expr: Expression,
        operator: Decorated<BinaryOperator>,
        rhs_expr: Expression,
    ) -> BinaryOp {
        BinaryOp {
            lhs_expr,
            operator,
            rhs_expr,
        }
    }

    pub fn lhs_expr(&self) -> &Expression {
        &self.lhs_expr
    }

    pub fn rhs_expr(&self) -> &Expression {
        &self.rhs_expr
    }

    pub fn operator(&self) -> &Decorated<BinaryOperator> {
        &self.operator
    }
}

impl Despan for BinaryOp {
    fn despan(&mut self, input: &str) {
        self.lhs_expr.despan(input);
        self.operator.decor_mut().despan(input);
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
    intro: Decorated<ForIntro>,
    key_expr: Option<Expression>,
    value_expr: Expression,
    grouping: bool,
    cond: Option<Decorated<ForCond>>,
}

impl ForExpr {
    pub fn new(intro: Decorated<ForIntro>, value_expr: Expression) -> ForExpr {
        ForExpr {
            intro,
            key_expr: None,
            value_expr,
            grouping: false,
            cond: None,
        }
    }

    pub fn intro(&self) -> &Decorated<ForIntro> {
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

    pub fn cond(&self) -> Option<&Decorated<ForCond>> {
        self.cond.as_ref()
    }

    pub fn set_cond(&mut self, cond: Decorated<ForCond>) {
        self.cond = Some(cond);
    }
}

impl Despan for ForExpr {
    fn despan(&mut self, input: &str) {
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
        let intro = expr.intro.into_inner();
        expr::ForExpr {
            key_var: intro.key_var.map(Decorated::into_inner),
            value_var: intro.value_var.into_inner(),
            collection_expr: intro.collection_expr.into(),
            key_expr: expr.key_expr.map(Into::into),
            value_expr: expr.value_expr.into(),
            grouping: expr.grouping,
            cond_expr: expr.cond.map(|cond| cond.into_inner().expr.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForIntro {
    key_var: Option<Decorated<Identifier>>,
    value_var: Decorated<Identifier>,
    collection_expr: Expression,
}

impl ForIntro {
    pub fn new(value_var: Decorated<Identifier>, collection_expr: Expression) -> ForIntro {
        ForIntro {
            key_var: None,
            value_var,
            collection_expr,
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
}

impl ForCond {
    pub fn new(expr: Expression) -> ForCond {
        ForCond { expr }
    }

    pub fn expr(&self) -> &Expression {
        &self.expr
    }
}

impl Despan for ForCond {
    fn despan(&mut self, input: &str) {
        self.expr.despan(input);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Body {
    structures: Vec<Structure>,
}

impl Body {
    pub fn new(structures: Vec<Structure>) -> Body {
        Body { structures }
    }

    pub fn structures(&self) -> &[Structure] {
        &self.structures
    }
}

impl Despan for Body {
    fn despan(&mut self, input: &str) {
        for structure in &mut self.structures {
            structure.despan(input);
        }
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
        self.encode(&mut state)
    }
}

#[derive(Debug, Clone)]
pub enum Structure {
    Attribute(Decorated<Attribute>),
    Block(Decorated<Block>),
}

impl Despan for Structure {
    fn despan(&mut self, input: &str) {
        match self {
            Structure::Attribute(attr) => attr.despan(input),
            Structure::Block(block) => block.despan(input),
        }
    }
}

impl Decorate for Structure {
    fn decor(&self) -> &Decor {
        match self {
            Structure::Attribute(attr) => attr.decor(),
            Structure::Block(block) => block.decor(),
        }
    }

    fn decor_mut(&mut self) -> &mut Decor {
        match self {
            Structure::Attribute(attr) => attr.decor_mut(),
            Structure::Block(block) => block.decor_mut(),
        }
    }
}

impl Span for Structure {
    fn span(&self) -> Option<Range<usize>> {
        match self {
            Structure::Attribute(attr) => attr.span(),
            Structure::Block(block) => block.span(),
        }
    }

    fn set_span(&mut self, span: Range<usize>) {
        match self {
            Structure::Attribute(attr) => attr.set_span(span),
            Structure::Block(block) => block.set_span(span),
        }
    }
}

impl From<Structure> for structure::Structure {
    fn from(structure: Structure) -> Self {
        match structure {
            Structure::Attribute(attr) => structure::Structure::Attribute(attr.inner_into()),
            Structure::Block(block) => structure::Structure::Block(block.inner_into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    key: Decorated<Identifier>,
    expr: Expression,
}

impl Attribute {
    pub fn new(key: Decorated<Identifier>, expr: Expression) -> Attribute {
        Attribute { key, expr }
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
}

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

impl Despan for BlockLabel {
    fn despan(&mut self, input: &str) {
        match self {
            BlockLabel::Identifier(ident) => ident.decor_mut().despan(input),
            BlockLabel::String(expr) => expr.decor_mut().despan(input),
        }
    }
}

impl Decorate for BlockLabel {
    fn decor(&self) -> &Decor {
        match self {
            BlockLabel::Identifier(ident) => ident.decor(),
            BlockLabel::String(expr) => expr.decor(),
        }
    }

    fn decor_mut(&mut self) -> &mut Decor {
        match self {
            BlockLabel::Identifier(ident) => ident.decor_mut(),
            BlockLabel::String(expr) => expr.decor_mut(),
        }
    }
}

impl Span for BlockLabel {
    fn span(&self) -> Option<Range<usize>> {
        match self {
            BlockLabel::Identifier(ident) => ident.span(),
            BlockLabel::String(expr) => expr.span(),
        }
    }

    fn set_span(&mut self, span: Range<usize>) {
        match self {
            BlockLabel::Identifier(ident) => ident.set_span(span),
            BlockLabel::String(expr) => expr.set_span(span),
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
    Multiline(Decorated<Body>),
    Oneline(Box<Decorated<Attribute>>),
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
            BlockBody::Multiline(body) => body.inner_into(),
            BlockBody::Oneline(attr) => structure::Attribute::from(attr.into_inner()).into(),
            BlockBody::Empty(_) => structure::Body::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template {
    elements: Vec<Element>,
}

impl Template {
    pub fn new(elements: Vec<Element>) -> Template {
        Template { elements }
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
    Interpolation(Spanned<Interpolation>),
    Directive(Directive),
}

impl Despan for Element {
    fn despan(&mut self, input: &str) {
        match self {
            Element::Literal(_) => {}
            Element::Interpolation(interp) => interp.despan(input),
            Element::Directive(dir) => dir.despan(input),
        }
    }
}

impl Span for Element {
    fn span(&self) -> Option<Range<usize>> {
        match self {
            Element::Literal(lit) => lit.span(),
            Element::Interpolation(interp) => interp.span(),
            Element::Directive(dir) => dir.span(),
        }
    }

    fn set_span(&mut self, span: Range<usize>) {
        match self {
            Element::Literal(lit) => lit.set_span(span),
            Element::Interpolation(interp) => interp.set_span(span),
            Element::Directive(dir) => dir.set_span(span),
        }
    }
}

impl From<Element> for template::Element {
    fn from(element: Element) -> Self {
        match element {
            Element::Literal(lit) => template::Element::Literal(lit.to_string()),
            Element::Interpolation(interp) => template::Element::Interpolation(interp.inner_into()),
            Element::Directive(dir) => template::Element::Directive(dir.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpolation {
    expr: Expression,
    strip: StripMode,
}

impl Interpolation {
    pub fn new(expr: Expression, strip: StripMode) -> Interpolation {
        Interpolation { expr, strip }
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
    If(Spanned<IfDirective>),
    For(Spanned<ForDirective>),
}

impl Despan for Directive {
    fn despan(&mut self, input: &str) {
        match self {
            Directive::If(dir) => dir.despan(input),
            Directive::For(dir) => dir.despan(input),
        }
    }
}

impl Span for Directive {
    fn span(&self) -> Option<Range<usize>> {
        match self {
            Directive::If(dir) => dir.span(),
            Directive::For(dir) => dir.span(),
        }
    }

    fn set_span(&mut self, span: Range<usize>) {
        match self {
            Directive::If(dir) => dir.set_span(span),
            Directive::For(dir) => dir.set_span(span),
        }
    }
}

impl From<Directive> for template::Directive {
    fn from(dir: Directive) -> Self {
        match dir {
            Directive::If(dir) => template::Directive::If(dir.inner_into()),
            Directive::For(dir) => template::Directive::For(dir.inner_into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfDirective {
    if_expr: IfTemplateExpr,
    else_expr: Option<ElseTemplateExpr>,
    endif_expr: EndifTemplateExpr,
}

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
            true_template: dir.if_expr.template.inner_into(),
            false_template: dir.else_expr.map(|expr| expr.template.inner_into()),
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
    template: Spanned<Template>,
    strip: StripMode,
}

impl IfTemplateExpr {
    pub fn new(
        cond_expr: Expression,
        template: Spanned<Template>,
        strip: StripMode,
    ) -> IfTemplateExpr {
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

    pub fn template(&self) -> &Spanned<Template> {
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
    template: Spanned<Template>,
    strip: StripMode,
}

impl ElseTemplateExpr {
    pub fn new(template: Spanned<Template>, strip: StripMode) -> ElseTemplateExpr {
        ElseTemplateExpr {
            preamble: RawString::default(),
            trailing: RawString::default(),
            template,
            strip,
        }
    }

    pub fn template(&self) -> &Spanned<Template> {
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
}

impl ForDirective {
    pub fn new(for_expr: ForTemplateExpr, endfor_expr: EndforTemplateExpr) -> ForDirective {
        ForDirective {
            for_expr,
            endfor_expr,
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
            template: dir.for_expr.template.inner_into(),
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
    template: Spanned<Template>,
    strip: StripMode,
}

impl ForTemplateExpr {
    pub fn new(
        key_var: Option<Decorated<Identifier>>,
        value_var: Decorated<Identifier>,
        collection_expr: Expression,
        template: Spanned<Template>,
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

    pub fn template(&self) -> &Spanned<Template> {
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
