#![allow(missing_docs)]

use super::repr::{Formatted, Spanned};
use crate::expr::{self, BinaryOperator, HeredocStripMode, Object, UnaryOperator, Variable};
use crate::structure::{self, BlockLabel};
use crate::template::{self, StripMode};
use crate::{Identifier, Number};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Formatted<Expression>>),
    Object(Object<Formatted<ObjectKey>, Formatted<Expression>>),
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
            Expression::Array(array) => array
                .into_iter()
                .map(|v| Expression::from(v.into_value()))
                .collect(),
            Expression::Object(object) => object
                .into_iter()
                .map(|(k, v)| {
                    (
                        ObjectKey::from(k.into_value()),
                        Expression::from(v.into_value()),
                    )
                })
                .collect(),
            Expression::Template(template) => {
                expr::TemplateExpr::QuotedString(template.into()).into()
            }
            Expression::HeredocTemplate(heredoc) => expr::Heredoc::from(*heredoc).into(),
            Expression::Parenthesis(expr) => {
                expr::Expression::Parenthesis(Box::new(expr.into_value().into()))
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
    pub delimiter: Formatted<Identifier>,
    pub template: Spanned<Template>,
    pub strip: HeredocStripMode,
}

impl From<HeredocTemplate> for expr::Heredoc {
    fn from(heredoc: HeredocTemplate) -> Self {
        expr::Heredoc {
            delimiter: heredoc.delimiter.into_value(),
            template: heredoc.template.into_value().into(),
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
            cond_expr: cond.cond_expr.into_value().into(),
            true_expr: cond.true_expr.into_value().into(),
            false_expr: cond.false_expr.into_value().into(),
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
            args: call
                .args
                .into_iter()
                .map(|spanned| spanned.into_value().into())
                .collect(),
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
            expr: traversal.expr.into_value().into(),
            operators: traversal
                .operators
                .into_iter()
                .map(|spanned| spanned.into_value().into())
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
            expr: op.expr.into_value().into(),
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
            lhs_expr: op.lhs_expr.into_value().into(),
            operator: op.operator.into_value(),
            rhs_expr: op.rhs_expr.into_value().into(),
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
            collection_expr: expr.collection_expr.into_value().into(),
            key_expr: expr.key_expr.map(|spanned| spanned.into_value().into()),
            value_expr: expr.value_expr.into_value().into(),
            grouping: expr.grouping,
            cond_expr: expr.cond_expr.map(|spanned| spanned.into_value().into()),
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
            expr: attr.expr.into_value().into(),
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
            BlockBody::Multiline(body) => body.into_value().into(),
            BlockBody::Oneline(attr) => attr
                .into_value()
                .map(|attr| structure::Attribute::from(attr).into())
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template {
    pub elements: Vec<Spanned<Element>>,
}

impl From<Template> for template::Template {
    fn from(template: Template) -> Self {
        template::Template::from_iter(
            template
                .elements
                .into_iter()
                .map(|spanned| template::Element::from(spanned.into_value())),
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
    pub expr: Formatted<Expression>,
    pub strip: StripMode,
}

impl From<Interpolation> for template::Interpolation {
    fn from(interp: Interpolation) -> Self {
        template::Interpolation {
            expr: interp.expr.into_value().into(),
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
    pub cond_expr: Formatted<Expression>,
    pub true_template: Spanned<Template>,
    pub false_template: Option<Spanned<Template>>,
    pub if_strip: StripMode,
    pub else_strip: StripMode,
    pub endif_strip: StripMode,
}

impl From<IfDirective> for template::IfDirective {
    fn from(dir: IfDirective) -> Self {
        template::IfDirective {
            cond_expr: dir.cond_expr.into_value().into(),
            true_template: dir.true_template.into_value().into(),
            false_template: dir
                .false_template
                .map(|spanned| spanned.into_value().into()),
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
    pub template: Spanned<Template>,
    pub for_strip: StripMode,
    pub endfor_strip: StripMode,
}

impl From<ForDirective> for template::ForDirective {
    fn from(dir: ForDirective) -> Self {
        template::ForDirective {
            key_var: dir.key_var.map(|spanned| spanned.into_value()),
            value_var: dir.value_var.into_value(),
            collection_expr: dir.collection_expr.into_value().into(),
            template: dir.template.into_value().into(),
            for_strip: dir.for_strip,
            endfor_strip: dir.endfor_strip,
        }
    }
}
