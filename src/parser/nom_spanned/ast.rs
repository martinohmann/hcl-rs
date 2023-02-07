#![allow(missing_docs)]

use super::repr::{Decor, Formatted, RawString, Spanned};
use crate::expr::{self, BinaryOperator, HeredocStripMode, UnaryOperator, Variable};
use crate::structure::{self, BlockLabel};
use crate::template::{self, StripMode};
use crate::{Identifier, Number};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Box<Array>),
    Object(Box<Object>),
    Template(Template),
    HeredocTemplate(Box<HeredocTemplate>),
    Parenthesis(Box<Formatted<Expression>>),
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
            Expression::Array(array) => expr::Expression::Array((*array).into()),
            Expression::Object(object) => expr::Expression::Object((*object).into()),
            Expression::Template(template) => {
                expr::TemplateExpr::QuotedString(template.into()).into()
            }
            Expression::HeredocTemplate(heredoc) => expr::Heredoc::from(*heredoc).into(),
            Expression::Parenthesis(expr) => {
                expr::Expression::Parenthesis(Box::new(expr.value_into()))
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Array {
    values: Vec<Formatted<Expression>>,
    trailing: RawString,
    trailing_comma: bool,
}

impl Array {
    pub fn new(values: Vec<Formatted<Expression>>) -> Array {
        Array {
            values,
            trailing: RawString::default(),
            trailing_comma: false,
        }
    }

    pub fn values(&self) -> &[Formatted<Expression>] {
        &self.values
    }

    pub fn items_mut(&mut self) -> &mut [Formatted<Expression>] {
        &mut self.values
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into()
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
        array
            .values
            .into_iter()
            .map(Formatted::value_into)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    items: Vec<ObjectItem>,
    trailing: RawString,
}

impl Object {
    pub fn new(items: Vec<ObjectItem>) -> Object {
        Object {
            items,
            trailing: RawString::default(),
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
        self.trailing = trailing.into()
    }
}

impl From<Object> for expr::Object<expr::ObjectKey, expr::Expression> {
    fn from(object: Object) -> Self {
        object
            .items
            .into_iter()
            .map(|item| (item.key.into(), item.value.value_into()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectItem {
    pub(crate) key: ObjectKey,
    pub(crate) key_value_separator: ObjectKeyValueSeparator,
    pub(crate) value: Formatted<Expression>,
    pub(crate) value_terminator: ObjectValueTerminator,
    pub(crate) decor: Decor,
    pub(crate) span: Option<Range<usize>>,
}

impl ObjectItem {
    pub fn new(key: ObjectKey, value: Formatted<Expression>) -> ObjectItem {
        ObjectItem {
            key,
            key_value_separator: ObjectKeyValueSeparator::Equals,
            value,
            value_terminator: ObjectValueTerminator::Newline,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn key(&self) -> &ObjectKey {
        &self.key
    }

    pub fn key_mut(&mut self) -> &mut ObjectKey {
        &mut self.key
    }

    pub fn value(&self) -> &Formatted<Expression> {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Formatted<Expression> {
        &mut self.value
    }

    pub fn into_key(self) -> ObjectKey {
        self.key
    }

    pub fn into_value(self) -> Formatted<Expression> {
        self.value
    }

    pub fn into_key_value(self) -> (ObjectKey, Formatted<Expression>) {
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

    pub fn decor(&self) -> &Decor {
        &self.decor
    }

    pub fn decor_mut(&mut self) -> &mut Decor {
        &mut self.decor
    }

    pub(crate) fn set_span(&mut self, span: Range<usize>) {
        self.span = Some(span);
    }

    pub fn span(&self) -> Option<Range<usize>> {
        self.span.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKey {
    Identifier(Formatted<Identifier>),
    Expression(Formatted<Expression>),
}

impl ObjectKey {
    pub fn decor(&self) -> &Decor {
        match self {
            ObjectKey::Identifier(ident) => ident.decor(),
            ObjectKey::Expression(expr) => expr.decor(),
        }
    }

    pub fn decor_mut(&mut self) -> &mut Decor {
        match self {
            ObjectKey::Identifier(ident) => ident.decor_mut(),
            ObjectKey::Expression(expr) => expr.decor_mut(),
        }
    }

    pub(crate) fn set_span(&mut self, span: Range<usize>) {
        match self {
            ObjectKey::Identifier(ident) => ident.set_span(span),
            ObjectKey::Expression(expr) => expr.set_span(span),
        }
    }

    pub fn span(&self) -> Option<Range<usize>> {
        match self {
            ObjectKey::Identifier(ident) => ident.span(),
            ObjectKey::Expression(expr) => expr.span(),
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
            ObjectKey::Expression(expr) => expr::ObjectKey::Expression(expr.value_into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeredocTemplate {
    pub delimiter: Formatted<Identifier>,
    pub template: Template,
    pub strip: HeredocStripMode,
}

impl From<HeredocTemplate> for expr::Heredoc {
    fn from(heredoc: HeredocTemplate) -> Self {
        expr::Heredoc {
            delimiter: heredoc.delimiter.into_value(),
            template: heredoc.template.into(),
            strip: heredoc.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    pub cond_expr: Formatted<Expression>,
    pub true_expr: Formatted<Expression>,
    pub false_expr: Formatted<Expression>,
}

impl From<Conditional> for expr::Conditional {
    fn from(cond: Conditional) -> Self {
        expr::Conditional {
            cond_expr: cond.cond_expr.value_into(),
            true_expr: cond.true_expr.value_into(),
            false_expr: cond.false_expr.value_into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncCall {
    pub name: Formatted<Identifier>,
    pub args: Vec<Formatted<Expression>>,
    pub expand_final: bool,
}

impl From<FuncCall> for expr::FuncCall {
    fn from(call: FuncCall) -> Self {
        expr::FuncCall {
            name: call.name.into_value(),
            args: call.args.into_iter().map(Formatted::value_into).collect(),
            expand_final: call.expand_final,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traversal {
    pub expr: Formatted<Expression>,
    pub operators: Vec<Formatted<TraversalOperator>>,
}

impl From<Traversal> for expr::Traversal {
    fn from(traversal: Traversal) -> Self {
        expr::Traversal {
            expr: traversal.expr.value_into(),
            operators: traversal
                .operators
                .into_iter()
                .map(Formatted::value_into)
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
    pub operator: Formatted<UnaryOperator>,
    pub expr: Formatted<Expression>,
}

impl From<UnaryOp> for expr::UnaryOp {
    fn from(op: UnaryOp) -> Self {
        expr::UnaryOp {
            operator: op.operator.into_value(),
            expr: op.expr.value_into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    pub lhs_expr: Formatted<Expression>,
    pub operator: Formatted<BinaryOperator>,
    pub rhs_expr: Formatted<Expression>,
}

impl From<BinaryOp> for expr::BinaryOp {
    fn from(op: BinaryOp) -> Self {
        expr::BinaryOp {
            lhs_expr: op.lhs_expr.value_into(),
            operator: op.operator.into_value(),
            rhs_expr: op.rhs_expr.value_into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForExpr {
    pub key_var: Option<Formatted<Identifier>>,
    pub value_var: Formatted<Identifier>,
    pub collection_expr: Formatted<Expression>,
    pub key_expr: Option<Formatted<Expression>>,
    pub value_expr: Formatted<Expression>,
    pub grouping: bool,
    pub cond_expr: Option<Formatted<Expression>>,
}

impl From<ForExpr> for expr::ForExpr {
    fn from(expr: ForExpr) -> Self {
        expr::ForExpr {
            key_var: expr.key_var.map(|spanned| spanned.into_value()),
            value_var: expr.value_var.into_value(),
            collection_expr: expr.collection_expr.value_into(),
            key_expr: expr.key_expr.map(Formatted::value_into),
            value_expr: expr.value_expr.value_into(),
            grouping: expr.grouping,
            cond_expr: expr.cond_expr.map(Formatted::value_into),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Body {
    pub structures: Vec<Formatted<Structure>>,
}

impl From<Body> for structure::Body {
    fn from(body: Body) -> Self {
        structure::Body::from_iter(body.structures.into_iter().map(Formatted::into_value))
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
    pub key: Formatted<Identifier>,
    pub expr: Formatted<Expression>,
}

impl From<Attribute> for structure::Attribute {
    fn from(attr: Attribute) -> Self {
        structure::Attribute {
            key: attr.key.into_value(),
            expr: attr.expr.value_into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub identifier: Formatted<Identifier>,
    pub labels: Vec<Formatted<BlockLabel>>,
    pub body: BlockBody,
}

impl From<Block> for structure::Block {
    fn from(block: Block) -> Self {
        structure::Block {
            identifier: block.identifier.into_value(),
            labels: block
                .labels
                .into_iter()
                .map(Formatted::into_value)
                .collect(),
            body: block.body.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockBody {
    Multiline(Formatted<Body>),
    Oneline(Formatted<Box<Option<Attribute>>>),
}

impl From<BlockBody> for structure::Body {
    fn from(body: BlockBody) -> Self {
        match body {
            BlockBody::Multiline(body) => body.value_into(),
            BlockBody::Oneline(attr) => attr
                .into_value()
                .map(|attr| structure::Attribute::from(attr).into())
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template {
    pub elements: Vec<Element>,
    pub(crate) span: Option<Range<usize>>,
}

impl Template {
    pub fn new(elements: Vec<Element>) -> Template {
        Template {
            elements,
            span: None,
        }
    }

    pub(crate) fn set_span(&mut self, span: Range<usize>) {
        self.span = Some(span);
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
    Interpolation(Interpolation),
    Directive(Directive),
}

impl Element {
    pub(crate) fn set_span(&mut self, span: Range<usize>) {
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
            Element::Interpolation(interp) => template::Element::Interpolation(interp.into()),
            Element::Directive(dir) => template::Element::Directive(dir.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpolation {
    pub expr: Formatted<Expression>,
    pub strip: StripMode,
    pub(crate) span: Option<Range<usize>>,
}

impl Interpolation {
    pub(crate) fn set_span(&mut self, span: Range<usize>) {
        self.span = Some(span);
    }
}

impl From<Interpolation> for template::Interpolation {
    fn from(interp: Interpolation) -> Self {
        template::Interpolation {
            expr: interp.expr.value_into(),
            strip: interp.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    If(IfDirective),
    For(ForDirective),
}

impl Directive {
    pub(crate) fn set_span(&mut self, span: Range<usize>) {
        match self {
            Directive::If(dir) => dir.set_span(span),
            Directive::For(dir) => dir.set_span(span),
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
    pub cond_expr: Formatted<Expression>,
    pub true_template: Template,
    pub false_template: Option<Template>,
    pub if_strip: StripMode,
    pub else_strip: StripMode,
    pub endif_strip: StripMode,
    pub(crate) span: Option<Range<usize>>,
}

impl IfDirective {
    pub(crate) fn set_span(&mut self, span: Range<usize>) {
        self.span = Some(span);
    }
}

impl From<IfDirective> for template::IfDirective {
    fn from(dir: IfDirective) -> Self {
        template::IfDirective {
            cond_expr: dir.cond_expr.value_into(),
            true_template: dir.true_template.into(),
            false_template: dir.false_template.map(Into::into),
            if_strip: dir.if_strip,
            else_strip: dir.else_strip,
            endif_strip: dir.endif_strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForDirective {
    pub key_var: Option<Formatted<Identifier>>,
    pub value_var: Formatted<Identifier>,
    pub collection_expr: Formatted<Expression>,
    pub template: Template,
    pub for_strip: StripMode,
    pub endfor_strip: StripMode,
    pub(crate) span: Option<Range<usize>>,
}

impl ForDirective {
    pub(crate) fn set_span(&mut self, span: Range<usize>) {
        self.span = Some(span);
    }
}

impl From<ForDirective> for template::ForDirective {
    fn from(dir: ForDirective) -> Self {
        template::ForDirective {
            key_var: dir.key_var.map(Formatted::into_value),
            value_var: dir.value_var.into_value(),
            collection_expr: dir.collection_expr.value_into(),
            template: dir.template.into(),
            for_strip: dir.for_strip,
            endfor_strip: dir.endfor_strip,
        }
    }
}
