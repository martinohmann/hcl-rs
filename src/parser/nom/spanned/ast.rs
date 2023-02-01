#![allow(missing_docs)]

use super::span::LocatedSpan;
use crate::expr::{self, BinaryOperator, HeredocStripMode, Object, UnaryOperator, Variable};
use crate::structure::{self, BlockLabel};
use crate::template::{self, StripMode};
use crate::{Identifier, Number};
use kstring::KString;
use std::ops::Range;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalString(pub(crate) KString);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawString(RawStringInner);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawStringInner {
    Empty,
    Spanned(Range<usize>),
    Explicit(InternalString),
}

impl RawString {
    pub(crate) fn from_span(span: Range<usize>) -> Self {
        if span.is_empty() {
            RawString(RawStringInner::Empty)
        } else {
            RawString(RawStringInner::Spanned(span))
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.0 {
            RawStringInner::Empty => Some(""),
            RawStringInner::Explicit(s) => Some(s.0.as_str()),
            RawStringInner::Spanned(_) => None,
        }
    }
}

impl Default for RawString {
    fn default() -> Self {
        RawString(RawStringInner::Empty)
    }
}

impl From<Range<usize>> for RawString {
    fn from(span: Range<usize>) -> Self {
        RawString::from_span(span)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Decor {
    pub prefix: Option<RawString>,
    pub suffix: Option<RawString>,
}

impl Decor {
    pub fn new(prefix: impl Into<RawString>, suffix: impl Into<RawString>) -> Decor {
        Decor {
            prefix: Some(prefix.into()),
            suffix: Some(suffix.into()),
        }
    }

    pub fn from_prefix(prefix: impl Into<RawString>) -> Decor {
        Decor {
            prefix: Some(prefix.into()),
            suffix: None,
        }
    }

    pub fn from_suffix(suffix: impl Into<RawString>) -> Decor {
        Decor {
            prefix: None,
            suffix: Some(suffix.into()),
        }
    }

    pub fn set_prefix(&mut self, prefix: impl Into<RawString>) {
        self.prefix = Some(prefix.into());
    }

    pub fn set_suffix(&mut self, suffix: impl Into<RawString>) {
        self.suffix = Some(suffix.into());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    value: T,
    span: Range<usize>,
    decor: Decor,
}

impl<T> Node<T> {
    pub fn new(value: T, span: Range<usize>) -> Node<T> {
        Node::new_with_decor(value, span, Decor::default())
    }

    pub fn new_with_decor(value: T, span: Range<usize>, decor: Decor) -> Node<T> {
        Node { value, span, decor }
    }

    pub fn map_value<F, U>(self, f: F) -> Node<U>
    where
        F: FnOnce(T) -> U,
    {
        Node {
            value: f(self.value),
            span: self.span,
            decor: self.decor,
        }
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Node<Expression>>),
    Object(Object<Node<ObjectKey>, Node<Expression>>),
    Template(Template),
    HeredocTemplate(Box<HeredocTemplate>),
    Parenthesis(Box<Node<Expression>>),
    Variable(Variable),
    Conditional(Box<Conditional>),
    FuncCall(Box<FuncCall>),
    Traversal(Box<Traversal>),
    Operation(Box<Operation>),
    ForExpr(Box<ForExpr>),
}

impl From<Expression> for expr::Expression {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::Null => expr::Expression::Null,
            Expression::Bool(b) => expr::Expression::Bool(b),
            Expression::Number(n) => expr::Expression::Number(n),
            Expression::String(s) => expr::Expression::String(s),
            Expression::Array(array) => array
                .into_iter()
                .map(|v| Expression::from(v.value))
                .collect(),
            Expression::Object(object) => object
                .into_iter()
                .map(|(k, v)| (ObjectKey::from(k.value), Expression::from(v.value)))
                .collect(),
            Expression::Template(template) => {
                expr::TemplateExpr::QuotedString(template.into()).into()
            }
            Expression::HeredocTemplate(heredoc) => expr::Heredoc::from(*heredoc).into(),
            Expression::Parenthesis(expr) => {
                expr::Expression::Parenthesis(Box::new(expr.value.into()))
            }
            Expression::Variable(var) => expr::Expression::Variable(var),
            Expression::ForExpr(expr) => expr::ForExpr::from(*expr).into(),
            Expression::Conditional(cond) => expr::Conditional::from(*cond).into(),
            Expression::FuncCall(call) => expr::FuncCall::from(*call).into(),
            Expression::Operation(op) => expr::Operation::from(*op).into(),
            Expression::Traversal(traversal) => expr::Traversal::from(*traversal).into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKey {
    Identifier(Identifier),
    Expression(Expression),
}

impl From<ObjectKey> for expr::ObjectKey {
    fn from(key: ObjectKey) -> Self {
        match key {
            ObjectKey::Identifier(ident) => expr::ObjectKey::Identifier(ident),
            ObjectKey::Expression(expr) => expr::ObjectKey::Expression(expr.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeredocTemplate {
    pub delimiter: Node<Identifier>,
    pub template: Node<String>,
    pub strip: HeredocStripMode,
}

impl From<HeredocTemplate> for expr::Heredoc {
    fn from(heredoc: HeredocTemplate) -> Self {
        expr::Heredoc {
            delimiter: heredoc.delimiter.value,
            template: heredoc.template.value,
            strip: heredoc.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    pub cond_expr: Node<Expression>,
    pub true_expr: Node<Expression>,
    pub false_expr: Node<Expression>,
}

impl From<Conditional> for expr::Conditional {
    fn from(cond: Conditional) -> Self {
        expr::Conditional {
            cond_expr: cond.cond_expr.value.into(),
            true_expr: cond.true_expr.value.into(),
            false_expr: cond.false_expr.value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncCall {
    pub name: Node<Identifier>,
    pub args: Vec<Node<Expression>>,
    pub expand_final: bool,
}

impl From<FuncCall> for expr::FuncCall {
    fn from(call: FuncCall) -> Self {
        expr::FuncCall {
            name: call.name.value,
            args: call
                .args
                .into_iter()
                .map(|spanned| spanned.value.into())
                .collect(),
            expand_final: call.expand_final,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traversal {
    pub expr: Node<Expression>,
    pub operators: Vec<Node<TraversalOperator>>,
}

impl From<Traversal> for expr::Traversal {
    fn from(traversal: Traversal) -> Self {
        expr::Traversal {
            expr: traversal.expr.value.into(),
            operators: traversal
                .operators
                .into_iter()
                .map(|spanned| spanned.value.into())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalOperator {
    AttrSplat,
    FullSplat,
    GetAttr(Identifier),
    Index(Expression),
    LegacyIndex(u64),
}

impl From<TraversalOperator> for expr::TraversalOperator {
    fn from(operator: TraversalOperator) -> Self {
        match operator {
            TraversalOperator::AttrSplat => expr::TraversalOperator::AttrSplat,
            TraversalOperator::FullSplat => expr::TraversalOperator::FullSplat,
            TraversalOperator::GetAttr(ident) => expr::TraversalOperator::GetAttr(ident),
            TraversalOperator::Index(expr) => expr::TraversalOperator::Index(expr.into()),
            TraversalOperator::LegacyIndex(index) => expr::TraversalOperator::LegacyIndex(index),
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
    pub operator: Node<UnaryOperator>,
    pub expr: Node<Expression>,
}

impl From<UnaryOp> for expr::UnaryOp {
    fn from(op: UnaryOp) -> Self {
        expr::UnaryOp {
            operator: op.operator.value,
            expr: op.expr.value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    pub lhs_expr: Node<Expression>,
    pub operator: Node<BinaryOperator>,
    pub rhs_expr: Node<Expression>,
}

impl From<BinaryOp> for expr::BinaryOp {
    fn from(op: BinaryOp) -> Self {
        expr::BinaryOp {
            lhs_expr: op.lhs_expr.value.into(),
            operator: op.operator.value,
            rhs_expr: op.rhs_expr.value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForExpr {
    pub key_var: Option<Node<Identifier>>,
    pub value_var: Node<Identifier>,
    pub collection_expr: Node<Expression>,
    pub key_expr: Option<Node<Expression>>,
    pub value_expr: Node<Expression>,
    pub grouping: bool,
    pub cond_expr: Option<Node<Expression>>,
}

impl From<ForExpr> for expr::ForExpr {
    fn from(expr: ForExpr) -> Self {
        expr::ForExpr {
            key_var: expr.key_var.map(|spanned| spanned.value),
            value_var: expr.value_var.value,
            collection_expr: expr.collection_expr.value.into(),
            key_expr: expr.key_expr.map(|spanned| spanned.value.into()),
            value_expr: expr.value_expr.value.into(),
            grouping: expr.grouping,
            cond_expr: expr.cond_expr.map(|spanned| spanned.value.into()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Body {
    pub structures: Vec<Node<Structure>>,
}

impl From<Body> for structure::Body {
    fn from(body: Body) -> Self {
        structure::Body::from_iter(body.structures.into_iter().map(Node::into_value))
    }
}

#[derive(Debug, Clone)]
pub enum Structure {
    Attribute(Attribute),
    Block(Block),
}

impl From<Structure> for structure::Structure {
    fn from(structure: Structure) -> Self {
        match structure {
            Structure::Attribute(attr) => structure::Structure::Attribute(attr.into()),
            Structure::Block(block) => structure::Structure::Block(block.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub key: Node<Identifier>,
    pub expr: Node<Expression>,
}

impl From<Attribute> for structure::Attribute {
    fn from(attr: Attribute) -> Self {
        structure::Attribute {
            key: attr.key.into_value(),
            expr: attr.expr.into_value().into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub identifier: Node<Identifier>,
    pub labels: Vec<Node<BlockLabel>>,
    pub body: Node<Body>,
}

impl From<Block> for structure::Block {
    fn from(block: Block) -> Self {
        structure::Block {
            identifier: block.identifier.into_value(),
            labels: block.labels.into_iter().map(Node::into_value).collect(),
            body: block.body.into_value().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub elements: Vec<Node<Element>>,
}

impl From<Template> for template::Template {
    fn from(template: Template) -> Self {
        template::Template::from_iter(
            template
                .elements
                .into_iter()
                .map(|spanned| template::Element::from(spanned.value)),
        )
    }
}

impl From<Template> for String {
    fn from(template: Template) -> Self {
        template::Template::from(template).to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    Literal(String),
    Interpolation(Interpolation),
    Directive(Directive),
}

impl From<Element> for template::Element {
    fn from(element: Element) -> Self {
        match element {
            Element::Literal(lit) => template::Element::Literal(lit),
            Element::Interpolation(interp) => template::Element::Interpolation(interp.into()),
            Element::Directive(dir) => template::Element::Directive(dir.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpolation {
    pub expr: Node<Expression>,
    pub strip: StripMode,
}

impl From<Interpolation> for template::Interpolation {
    fn from(interp: Interpolation) -> Self {
        template::Interpolation {
            expr: interp.expr.value.into(),
            strip: interp.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    If(IfDirective),
    For(ForDirective),
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
    pub cond_expr: Node<Expression>,
    pub true_template: Node<Template>,
    pub false_template: Option<Node<Template>>,
    pub if_strip: StripMode,
    pub else_strip: StripMode,
    pub endif_strip: StripMode,
}

impl From<IfDirective> for template::IfDirective {
    fn from(dir: IfDirective) -> Self {
        template::IfDirective {
            cond_expr: dir.cond_expr.value.into(),
            true_template: dir.true_template.value.into(),
            false_template: dir.false_template.map(|spanned| spanned.value.into()),
            if_strip: dir.if_strip,
            else_strip: dir.else_strip,
            endif_strip: dir.endif_strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForDirective {
    pub key_var: Option<Node<Identifier>>,
    pub value_var: Node<Identifier>,
    pub collection_expr: Node<Expression>,
    pub template: Node<Template>,
    pub for_strip: StripMode,
    pub endfor_strip: StripMode,
}

impl From<ForDirective> for template::ForDirective {
    fn from(dir: ForDirective) -> Self {
        template::ForDirective {
            key_var: dir.key_var.map(|spanned| spanned.value),
            value_var: dir.value_var.value,
            collection_expr: dir.collection_expr.value.into(),
            template: dir.template.value.into(),
            for_strip: dir.for_strip,
            endfor_strip: dir.endfor_strip,
        }
    }
}
