use crate::expr::{self, BinaryOperator, HeredocStripMode, Object, UnaryOperator, Variable};
use crate::structure::{self, BlockLabel};
use crate::template::{self, StripMode};
use crate::{Identifier, Number};
use nom_locate::LocatedSpan;
use std::borrow::Cow;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type Str<'a> = Cow<'a, str>;

#[derive(Default)]
pub struct Decor<'a> {
    pub prefix: Option<Str<'a>>,
    pub suffix: Option<Str<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<'a, T> {
    pub value: T,
    pub start: Span<'a>,
    pub end: Span<'a>,
    // pub decor: Decor<'a>,
}

impl<'a, T> Spanned<'a, T> {
    pub fn map_value<F, U>(self, f: F) -> Spanned<'a, U>
    where
        F: FnOnce(T) -> U,
    {
        Spanned {
            value: f(self.value),
            start: self.start,
            end: self.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression<'a> {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Spanned<'a, Expression<'a>>>),
    Object(Object<Spanned<'a, ObjectKey<'a>>, Spanned<'a, Expression<'a>>>),
    TemplateExpr(Box<TemplateExpr>),
    Parenthesis(Box<Spanned<'a, Expression<'a>>>),
    Variable(Variable),
    Conditional(Box<Conditional<'a>>),
    FuncCall(Box<FuncCall<'a>>),
    Traversal(Box<Traversal<'a>>),
    Operation(Box<Operation<'a>>),
    ForExpr(Box<ForExpr<'a>>),
}

impl<'a> From<Expression<'a>> for expr::Expression {
    fn from(expr: Expression<'a>) -> Self {
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
            Expression::TemplateExpr(expr) => expr::TemplateExpr::from(*expr).into(),
            Expression::Parenthesis(expr) => {
                expr::Expression::Parenthesis(Box::new(expr::Expression::from((*expr).value)))
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
pub enum ObjectKey<'a> {
    Identifier(Identifier),
    Expression(Expression<'a>),
}

impl<'a> From<ObjectKey<'a>> for expr::ObjectKey {
    fn from(key: ObjectKey<'a>) -> Self {
        match key {
            ObjectKey::Identifier(ident) => expr::ObjectKey::Identifier(ident),
            ObjectKey::Expression(expr) => expr::ObjectKey::Expression(expr.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateExpr {
    QuotedString(String),
    Heredoc(Heredoc),
}

impl From<TemplateExpr> for expr::TemplateExpr {
    fn from(expr: TemplateExpr) -> Self {
        match expr {
            TemplateExpr::QuotedString(s) => expr::TemplateExpr::QuotedString(s),
            TemplateExpr::Heredoc(hd) => expr::TemplateExpr::Heredoc(hd.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heredoc {
    pub delimiter: Identifier,
    pub template: String,
    pub strip: HeredocStripMode,
}

impl From<Heredoc> for expr::Heredoc {
    fn from(heredoc: Heredoc) -> Self {
        expr::Heredoc {
            delimiter: heredoc.delimiter,
            template: heredoc.template,
            strip: heredoc.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional<'a> {
    pub cond_expr: Expression<'a>,
    pub true_expr: Spanned<'a, Expression<'a>>,
    pub false_expr: Spanned<'a, Expression<'a>>,
}

impl<'a> From<Conditional<'a>> for expr::Conditional {
    fn from(cond: Conditional<'a>) -> Self {
        expr::Conditional {
            cond_expr: cond.cond_expr.into(),
            true_expr: cond.true_expr.value.into(),
            false_expr: cond.false_expr.value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncCall<'a> {
    pub name: Spanned<'a, Identifier>,
    pub args: Vec<Spanned<'a, Expression<'a>>>,
    pub expand_final: bool,
}

impl<'a> From<FuncCall<'a>> for expr::FuncCall {
    fn from(call: FuncCall<'a>) -> Self {
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
pub struct Traversal<'a> {
    pub expr: Expression<'a>,
    pub operators: Vec<Spanned<'a, TraversalOperator<'a>>>,
}

impl<'a> From<Traversal<'a>> for expr::Traversal {
    fn from(traversal: Traversal<'a>) -> Self {
        expr::Traversal {
            expr: traversal.expr.into(),
            operators: traversal
                .operators
                .into_iter()
                .map(|spanned| spanned.value.into())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalOperator<'a> {
    AttrSplat,
    FullSplat,
    GetAttr(Identifier),
    Index(Expression<'a>),
    LegacyIndex(u64),
}

impl<'a> From<TraversalOperator<'a>> for expr::TraversalOperator {
    fn from(operator: TraversalOperator<'a>) -> Self {
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
pub enum Operation<'a> {
    Unary(UnaryOp<'a>),
    Binary(BinaryOp<'a>),
}

impl<'a> From<Operation<'a>> for expr::Operation {
    fn from(op: Operation<'a>) -> Self {
        match op {
            Operation::Unary(unary) => expr::Operation::Unary(unary.into()),
            Operation::Binary(binary) => expr::Operation::Binary(binary.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOp<'a> {
    pub expr: Expression<'a>,
    pub operator: UnaryOperator,
}

impl<'a> From<UnaryOp<'a>> for expr::UnaryOp {
    fn from(op: UnaryOp<'a>) -> Self {
        expr::UnaryOp {
            operator: op.operator,
            expr: op.expr.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp<'a> {
    pub lhs_expr: Expression<'a>,
    pub operator: BinaryOperator,
    pub rhs_expr: Spanned<'a, Expression<'a>>,
}

impl<'a> From<BinaryOp<'a>> for expr::BinaryOp {
    fn from(op: BinaryOp<'a>) -> Self {
        expr::BinaryOp {
            lhs_expr: op.lhs_expr.into(),
            operator: op.operator,
            rhs_expr: op.rhs_expr.value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForExpr<'a> {
    pub key_var: Option<Spanned<'a, Identifier>>,
    pub value_var: Spanned<'a, Identifier>,
    pub collection_expr: Spanned<'a, Expression<'a>>,
    pub key_expr: Option<Spanned<'a, Expression<'a>>>,
    pub value_expr: Spanned<'a, Expression<'a>>,
    pub grouping: bool,
    pub cond_expr: Option<Spanned<'a, Expression<'a>>>,
}

impl<'a> From<ForExpr<'a>> for expr::ForExpr {
    fn from(expr: ForExpr<'a>) -> Self {
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
pub struct Body<'a> {
    pub structures: Vec<Spanned<'a, Structure<'a>>>,
}

#[derive(Debug, Clone)]
pub enum Structure<'a> {
    Attribute(Attribute<'a>),
    Block(Block<'a>),
}

#[derive(Debug, Clone)]
pub struct Attribute<'a> {
    pub key: Spanned<'a, Identifier>,
    pub expr: Spanned<'a, Expression<'a>>,
}

#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub identifier: Spanned<'a, Identifier>,
    pub labels: Vec<Spanned<'a, BlockLabel>>,
    pub body: Spanned<'a, Body<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template<'a> {
    pub elements: Vec<Spanned<'a, Element<'a>>>,
}

impl<'a> From<Template<'a>> for template::Template {
    fn from(template: Template) -> Self {
        template::Template::from_iter(
            template
                .elements
                .into_iter()
                .map(|spanned| template::Element::from(spanned.value)),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element<'a> {
    Literal(String),
    Interpolation(Interpolation<'a>),
    Directive(Directive<'a>),
}

impl<'a> From<Element<'a>> for template::Element {
    fn from(element: Element) -> Self {
        match element {
            Element::Literal(lit) => template::Element::Literal(lit),
            Element::Interpolation(interp) => template::Element::Interpolation(interp.into()),
            Element::Directive(dir) => template::Element::Directive(dir.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpolation<'a> {
    pub expr: Spanned<'a, Expression<'a>>,
    pub strip: StripMode,
}

impl<'a> From<Interpolation<'a>> for template::Interpolation {
    fn from(interp: Interpolation<'a>) -> Self {
        template::Interpolation {
            expr: interp.expr.value.into(),
            strip: interp.strip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive<'a> {
    If(IfDirective<'a>),
    For(ForDirective<'a>),
}

impl<'a> From<Directive<'a>> for template::Directive {
    fn from(dir: Directive<'a>) -> Self {
        match dir {
            Directive::If(dir) => template::Directive::If(dir.into()),
            Directive::For(dir) => template::Directive::For(dir.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfDirective<'a> {
    pub cond_expr: Spanned<'a, Expression<'a>>,
    pub true_template: Spanned<'a, Template<'a>>,
    pub false_template: Option<Spanned<'a, Template<'a>>>,
    pub if_strip: StripMode,
    pub else_strip: StripMode,
    pub endif_strip: StripMode,
}

impl<'a> From<IfDirective<'a>> for template::IfDirective {
    fn from(dir: IfDirective<'a>) -> Self {
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
pub struct ForDirective<'a> {
    pub key_var: Option<Spanned<'a, Identifier>>,
    pub value_var: Spanned<'a, Identifier>,
    pub collection_expr: Spanned<'a, Expression<'a>>,
    pub template: Spanned<'a, Template<'a>>,
    pub for_strip: StripMode,
    pub endfor_strip: StripMode,
}

impl<'a> From<ForDirective<'a>> for template::ForDirective {
    fn from(dir: ForDirective<'a>) -> Self {
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
