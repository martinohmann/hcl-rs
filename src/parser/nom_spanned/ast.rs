#![allow(missing_docs)]

use super::repr::{Decor, Decorate, Decorated, RawString, Span, Spanned};
use crate::expr::{self, BinaryOperator, HeredocStripMode, UnaryOperator, Variable};
use crate::structure;
use crate::template::{self, StripMode};
use crate::{Identifier, Number};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Null;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Null(Decorated<Null>),
    Bool(Decorated<bool>),
    Number(Decorated<Number>),
    String(Decorated<String>),
    Array(Box<Decorated<Array>>),
    Object(Box<Decorated<Object>>),
    Template(Decorated<Template>),
    HeredocTemplate(Box<Decorated<HeredocTemplate>>),
    Parenthesis(Box<Decorated<Expression>>),
    Variable(Decorated<Variable>),
    Conditional(Box<Decorated<Conditional>>),
    FuncCall(Box<Decorated<FuncCall>>),
    Traversal(Box<Decorated<Traversal>>),
    Operation(Box<Decorated<Operation>>),
    ForExpr(Box<Decorated<ForExpr>>),
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
            Expression::Operation(op) => op.decor(),
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
            Expression::Operation(op) => op.decor_mut(),
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
            Expression::Operation(op) => op.span(),
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
            Expression::Operation(op) => op.set_span(span),
            Expression::Traversal(traversal) => traversal.set_span(span),
        }
    }
}

impl From<Expression> for expr::Expression {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::Null(_) => expr::Expression::Null,
            Expression::Bool(b) => expr::Expression::Bool(b.into_value()),
            Expression::Number(n) => expr::Expression::Number(n.into_value()),
            Expression::String(s) => expr::Expression::String(s.into_value()),
            Expression::Array(array) => expr::Expression::Array(array.value_into()),
            Expression::Object(object) => expr::Expression::Object(object.value_into()),
            Expression::Template(template) => {
                expr::TemplateExpr::QuotedString(template.value_into()).into()
            }
            Expression::HeredocTemplate(heredoc) => {
                expr::Heredoc::from(heredoc.into_value()).into()
            }
            Expression::Parenthesis(expr) => {
                expr::Expression::Parenthesis(Box::new(expr.value_into()))
            }
            Expression::Variable(var) => expr::Expression::Variable(var.into_value()),
            Expression::ForExpr(expr) => expr::ForExpr::from(expr.into_value()).into(),
            Expression::Conditional(cond) => expr::Conditional::from(cond.into_value()).into(),
            Expression::FuncCall(call) => expr::FuncCall::from(call.into_value()).into(),
            Expression::Operation(op) => expr::Operation::from(op.into_value()).into(),
            Expression::Traversal(traversal) => {
                expr::Traversal::from(traversal.into_value()).into()
            }
        }
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

    pub fn items_mut(&mut self) -> &mut [Expression] {
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

impl From<Array> for Vec<expr::Expression> {
    fn from(array: Array) -> Self {
        array.values.into_iter().map(Into::into).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    items: Vec<Decorated<ObjectItem>>,
    trailing: RawString,
}

impl Object {
    pub fn new(items: Vec<Decorated<ObjectItem>>) -> Object {
        Object {
            items,
            trailing: RawString::default(),
        }
    }

    pub fn items(&self) -> &[Decorated<ObjectItem>] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut [Decorated<ObjectItem>] {
        &mut self.items
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }
}

impl From<Object> for expr::Object<expr::ObjectKey, expr::Expression> {
    fn from(object: Object) -> Self {
        object
            .items
            .into_iter()
            .map(Decorated::into_value)
            .map(|item| (item.key.into(), item.value.into()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectItem {
    pub(crate) key: ObjectKey,
    pub(crate) key_value_separator: ObjectKeyValueSeparator,
    pub(crate) value: Expression,
    pub(crate) value_terminator: ObjectValueTerminator,
}

impl ObjectItem {
    pub fn new(key: ObjectKey, value: Expression) -> ObjectItem {
        ObjectItem {
            key,
            key_value_separator: ObjectKeyValueSeparator::Equals,
            value,
            value_terminator: ObjectValueTerminator::Newline,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKey {
    Identifier(Decorated<Identifier>),
    Expression(Expression),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectValueTerminator {
    None,
    Newline,
    Comma,
}

impl From<ObjectKey> for expr::ObjectKey {
    fn from(key: ObjectKey) -> Self {
        match key {
            ObjectKey::Identifier(ident) => expr::ObjectKey::Identifier(ident.into_value()),
            ObjectKey::Expression(expr) => expr::ObjectKey::Expression(expr.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeredocTemplate {
    pub delimiter: Decorated<Identifier>,
    pub template: Spanned<Template>,
    pub strip: HeredocStripMode,
}

impl From<HeredocTemplate> for expr::Heredoc {
    fn from(heredoc: HeredocTemplate) -> Self {
        expr::Heredoc {
            delimiter: heredoc.delimiter.into_value(),
            template: heredoc.template.value_into(),
            strip: heredoc.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    pub cond_expr: Expression,
    pub true_expr: Expression,
    pub false_expr: Expression,
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
    pub name: Decorated<Identifier>,
    pub args: Vec<Expression>,
    pub expand_final: bool,
}

impl From<FuncCall> for expr::FuncCall {
    fn from(call: FuncCall) -> Self {
        expr::FuncCall {
            name: call.name.into_value(),
            args: call.args.into_iter().map(Into::into).collect(),
            expand_final: call.expand_final,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traversal {
    pub expr: Expression,
    pub operators: Vec<Decorated<TraversalOperator>>,
}

impl From<Traversal> for expr::Traversal {
    fn from(traversal: Traversal) -> Self {
        expr::Traversal {
            expr: traversal.expr.into(),
            operators: traversal
                .operators
                .into_iter()
                .map(Decorated::value_into)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalOperator {
    AttrSplat,
    FullSplat,
    GetAttr(Decorated<Identifier>),
    Index(Expression),
    LegacyIndex(Decorated<u64>),
}

impl From<TraversalOperator> for expr::TraversalOperator {
    fn from(operator: TraversalOperator) -> Self {
        match operator {
            TraversalOperator::AttrSplat => expr::TraversalOperator::AttrSplat,
            TraversalOperator::FullSplat => expr::TraversalOperator::FullSplat,
            TraversalOperator::GetAttr(ident) => {
                expr::TraversalOperator::GetAttr(ident.into_value())
            }
            TraversalOperator::Index(expr) => expr::TraversalOperator::Index(expr.into()),
            TraversalOperator::LegacyIndex(index) => {
                expr::TraversalOperator::LegacyIndex(index.into_value())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Unary(UnaryOp),
    Binary(BinaryOp),
}

impl From<Operation> for expr::Operation {
    fn from(op: Operation) -> Self {
        match op {
            Operation::Unary(unary) => expr::Operation::Unary(unary.into()),
            Operation::Binary(binary) => expr::Operation::Binary(binary.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOp {
    pub operator: Spanned<UnaryOperator>,
    pub expr: Expression,
}

impl From<UnaryOp> for expr::UnaryOp {
    fn from(op: UnaryOp) -> Self {
        expr::UnaryOp {
            operator: op.operator.into_value(),
            expr: op.expr.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    pub lhs_expr: Expression,
    pub operator: Decorated<BinaryOperator>,
    pub rhs_expr: Expression,
}

impl From<BinaryOp> for expr::BinaryOp {
    fn from(op: BinaryOp) -> Self {
        expr::BinaryOp {
            lhs_expr: op.lhs_expr.into(),
            operator: op.operator.into_value(),
            rhs_expr: op.rhs_expr.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForExpr {
    pub(crate) prefix: RawString,
    pub key_var: Option<Decorated<Identifier>>,
    pub value_var: Decorated<Identifier>,
    pub collection_expr: Expression,
    pub key_expr: Option<Expression>,
    pub value_expr: Expression,
    pub grouping: bool,
    pub cond_expr: Option<Expression>,
}

impl From<ForExpr> for expr::ForExpr {
    fn from(expr: ForExpr) -> Self {
        expr::ForExpr {
            key_var: expr.key_var.map(Decorated::into_value),
            value_var: expr.value_var.into_value(),
            collection_expr: expr.collection_expr.into(),
            key_expr: expr.key_expr.map(Into::into),
            value_expr: expr.value_expr.into(),
            grouping: expr.grouping,
            cond_expr: expr.cond_expr.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Body {
    pub structures: Vec<Structure>,
}

impl From<Body> for structure::Body {
    fn from(body: Body) -> Self {
        structure::Body::from_iter(body.structures)
    }
}

#[derive(Debug, Clone)]
pub enum Structure {
    Attribute(Decorated<Attribute>),
    Block(Decorated<Block>),
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
            Structure::Attribute(attr) => structure::Structure::Attribute(attr.value_into()),
            Structure::Block(block) => structure::Structure::Block(block.value_into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub key: Decorated<Identifier>,
    pub expr: Expression,
}

impl Attribute {
    pub fn new(key: Decorated<Identifier>, expr: Expression) -> Attribute {
        Attribute { key, expr }
    }
}

impl From<Attribute> for structure::Attribute {
    fn from(attr: Attribute) -> Self {
        structure::Attribute {
            key: attr.key.into_value(),
            expr: attr.expr.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub identifier: Decorated<Identifier>,
    pub labels: Vec<BlockLabel>,
    pub body: BlockBody,
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
}

impl From<Block> for structure::Block {
    fn from(block: Block) -> Self {
        structure::Block {
            identifier: block.identifier.into_value(),
            labels: block.labels.into_iter().map(Into::into).collect(),
            body: block.body.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockLabel {
    Identifier(Decorated<Identifier>),
    String(Decorated<String>),
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
            BlockLabel::Identifier(ident) => structure::BlockLabel::Identifier(ident.into_value()),
            BlockLabel::String(expr) => structure::BlockLabel::String(expr.into_value()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockBody {
    Multiline(Decorated<Body>),
    Oneline(Box<Decorated<Attribute>>),
    Empty(RawString),
}

impl From<BlockBody> for structure::Body {
    fn from(body: BlockBody) -> Self {
        match body {
            BlockBody::Multiline(body) => body.value_into(),
            BlockBody::Oneline(attr) => structure::Attribute::from(attr.into_value()).into(),
            BlockBody::Empty(_) => structure::Body::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template {
    pub elements: Vec<Element>,
}

impl Template {
    pub fn new(elements: Vec<Element>) -> Template {
        Template { elements }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    Literal(Spanned<String>),
    Interpolation(Spanned<Interpolation>),
    Directive(Directive),
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
            Element::Literal(lit) => template::Element::Literal(lit.into_value()),
            Element::Interpolation(interp) => template::Element::Interpolation(interp.value_into()),
            Element::Directive(dir) => template::Element::Directive(dir.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpolation {
    pub expr: Expression,
    pub strip: StripMode,
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
            Directive::If(dir) => template::Directive::If(dir.value_into()),
            Directive::For(dir) => template::Directive::For(dir.value_into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfDirective {
    pub cond_expr: Expression,
    pub true_template: Spanned<Template>,
    pub false_template: Option<Spanned<Template>>,
    pub if_strip: StripMode,
    pub else_strip: StripMode,
    pub endif_strip: StripMode,
}

impl From<IfDirective> for template::IfDirective {
    fn from(dir: IfDirective) -> Self {
        template::IfDirective {
            cond_expr: dir.cond_expr.into(),
            true_template: dir.true_template.value_into(),
            false_template: dir.false_template.map(Spanned::value_into),
            if_strip: dir.if_strip,
            else_strip: dir.else_strip,
            endif_strip: dir.endif_strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForDirective {
    pub key_var: Option<Decorated<Identifier>>,
    pub value_var: Decorated<Identifier>,
    pub collection_expr: Expression,
    pub template: Spanned<Template>,
    pub for_strip: StripMode,
    pub endfor_strip: StripMode,
}

impl From<ForDirective> for template::ForDirective {
    fn from(dir: ForDirective) -> Self {
        template::ForDirective {
            key_var: dir.key_var.map(Decorated::into_value),
            value_var: dir.value_var.into_value(),
            collection_expr: dir.collection_expr.into(),
            template: dir.template.value_into(),
            for_strip: dir.for_strip,
            endfor_strip: dir.endfor_strip,
        }
    }
}
