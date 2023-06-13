//! HCL language item traversal.
//!
//! Each method of the [`Visit`] trait is a hook that can be overridden to customize the behavior
//! when visiting the corresponding type of language item. By default, every method recursively
//! visits the substructure of the AST by invoking the right visitor method of each of its fields.
//!
//! The API is modeled after [`syn::visit`](https://docs.rs/syn/latest/syn/visit/index.html). For a
//! mutable alternative, see [`hcl_edit::visit_mut`](crate::visit_mut).
//!
//! # Examples
//!
//! Collect all referenced variables from a HCL document:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use hcl_edit::expr::Expression;
//! use hcl_edit::structure::Body;
//! use hcl_edit::visit::{visit_expr, Visit};
//! use std::collections::HashSet;
//! use std::str::FromStr;
//!
//! #[derive(Default)]
//! struct VariableNameVisitor {
//!     variable_names: HashSet<String>,
//! }
//!
//! impl Visit for VariableNameVisitor {
//!     fn visit_expr(&mut self, expr: &Expression) {
//!         if let Expression::Variable(var) = expr {
//!             self.variable_names.insert(var.to_string());
//!         } else {
//!             // Recurse further down the AST.
//!             visit_expr(self, expr);
//!         }
//!     }
//! }
//!
//! let input = r#"
//!     // A service definition.
//!     service {
//!         fullname        = "${namespace}/${name}"
//!         health_endpoint = "${base_url}/health"
//!     }
//! "#;
//!
//! let body = input.parse::<Body>()?;
//!
//! let mut visitor = VariableNameVisitor::default();
//!
//! visitor.visit_body(&body);
//!
//! let expected = HashSet::from(["namespace".into(), "name".into(), "base_url".into()]);
//!
//! assert_eq!(visitor.variable_names, expected);
//! #   Ok(())
//! # }
//! ```

#![allow(missing_docs)]

use crate::expr::{
    Array, BinaryOp, BinaryOperator, Conditional, Expression, ForCond, ForExpr, ForIntro, FuncArgs,
    FuncCall, Null, Object, ObjectKey, ObjectValue, Parenthesis, Splat, Traversal,
    TraversalOperator, UnaryOp, UnaryOperator,
};
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use crate::template::{
    Directive, Element, ElseTemplateExpr, EndforTemplateExpr, EndifTemplateExpr, ForDirective,
    ForTemplateExpr, HeredocTemplate, IfDirective, IfTemplateExpr, Interpolation, StringTemplate,
    Template,
};
use crate::{Decorated, Formatted, Ident, Number, Spanned};

macro_rules! empty_visit_methods {
    ($($name: ident => $t: ty),+ $(,)?) => {
        $(
            fn $name(&mut self, node: &$t) {
                let _ = node;
            }
        )*
    };
}

macro_rules! visit_methods {
    ($($name: ident => $t: ty),+ $(,)?) => {
        $(
            fn $name(&mut self, node: &$t) {
                $name(self, node);
            }
        )*
    };
}

/// Traversal to walk a shared borrow of an HCL language item.
///
/// See the [module documentation](crate::visit) for details.
pub trait Visit {
    empty_visit_methods! {
        visit_ident => Decorated<Ident>,
        visit_null => Decorated<Null>,
        visit_bool => Decorated<bool>,
        visit_u64 => Decorated<u64>,
        visit_number => Formatted<Number>,
        visit_string => Decorated<String>,
        visit_splat => Decorated<Splat>,
        visit_literal => Spanned<String>,
        visit_unary_operator => Spanned<UnaryOperator>,
        visit_binary_operator => Spanned<BinaryOperator>,
        visit_endif_template_expr => EndifTemplateExpr,
        visit_endfor_template_expr => EndforTemplateExpr,
    }

    visit_methods! {
        visit_body => Body,
        visit_structure => Structure,
        visit_attr => Attribute,
        visit_block => Block,
        visit_block_label => BlockLabel,
        visit_expr => Expression,
        visit_array => Array,
        visit_object => Object,
        visit_object_key => ObjectKey,
        visit_object_value => ObjectValue,
        visit_parenthesis => Parenthesis,
        visit_conditional => Conditional,
        visit_unary_op => UnaryOp,
        visit_binary_op => BinaryOp,
        visit_traversal => Traversal,
        visit_traversal_operator => TraversalOperator,
        visit_func_call => FuncCall,
        visit_func_args => FuncArgs,
        visit_for_expr => ForExpr,
        visit_for_intro => ForIntro,
        visit_for_cond => ForCond,
        visit_string_template => StringTemplate,
        visit_heredoc_template => HeredocTemplate,
        visit_template => Template,
        visit_element => Element,
        visit_interpolation => Interpolation,
        visit_directive => Directive,
        visit_if_directive => IfDirective,
        visit_for_directive => ForDirective,
        visit_if_template_expr => IfTemplateExpr,
        visit_else_template_expr => ElseTemplateExpr,
        visit_for_template_expr => ForTemplateExpr,
    }

    fn visit_object_item(&mut self, key: &ObjectKey, value: &ObjectValue) {
        visit_object_item(self, key, value);
    }
}

pub fn visit_body<V>(v: &mut V, node: &Body)
where
    V: Visit + ?Sized,
{
    for structure in node.iter() {
        v.visit_structure(structure);
    }
}

pub fn visit_structure<V>(v: &mut V, node: &Structure)
where
    V: Visit + ?Sized,
{
    match node {
        Structure::Attribute(attr) => v.visit_attr(attr),
        Structure::Block(block) => v.visit_block(block),
    }
}

pub fn visit_attr<V>(v: &mut V, node: &Attribute)
where
    V: Visit + ?Sized,
{
    v.visit_ident(&node.key);
    v.visit_expr(&node.value);
}

pub fn visit_block<V>(v: &mut V, node: &Block)
where
    V: Visit + ?Sized,
{
    v.visit_ident(&node.ident);
    for label in &node.labels {
        v.visit_block_label(label);
    }
    v.visit_body(&node.body);
}

pub fn visit_block_label<V>(v: &mut V, node: &BlockLabel)
where
    V: Visit + ?Sized,
{
    match node {
        BlockLabel::String(string) => v.visit_string(string),
        BlockLabel::Ident(ident) => v.visit_ident(ident),
    }
}

pub fn visit_expr<V>(v: &mut V, node: &Expression)
where
    V: Visit + ?Sized,
{
    match node {
        Expression::Null(null) => v.visit_null(null),
        Expression::Bool(b) => v.visit_bool(b),
        Expression::Number(number) => v.visit_number(number),
        Expression::String(string) => v.visit_string(string),
        Expression::Array(array) => v.visit_array(array),
        Expression::Object(object) => v.visit_object(object),
        Expression::Template(template) => v.visit_string_template(template),
        Expression::HeredocTemplate(template) => v.visit_heredoc_template(template),
        Expression::Parenthesis(parens) => v.visit_parenthesis(parens),
        Expression::Variable(var) => v.visit_ident(var),
        Expression::ForExpr(for_expr) => v.visit_for_expr(for_expr),
        Expression::Conditional(conditional) => v.visit_conditional(conditional),
        Expression::FuncCall(func_call) => v.visit_func_call(func_call),
        Expression::UnaryOp(unary_op) => v.visit_unary_op(unary_op),
        Expression::BinaryOp(binary_op) => v.visit_binary_op(binary_op),
        Expression::Traversal(traversal) => v.visit_traversal(traversal),
    }
}

pub fn visit_array<V>(v: &mut V, node: &Array)
where
    V: Visit + ?Sized,
{
    for expr in node.iter() {
        v.visit_expr(expr);
    }
}

pub fn visit_object<V>(v: &mut V, node: &Object)
where
    V: Visit + ?Sized,
{
    for (key, value) in node.iter() {
        v.visit_object_item(key, value);
    }
}

pub fn visit_object_item<V>(v: &mut V, key: &ObjectKey, value: &ObjectValue)
where
    V: Visit + ?Sized,
{
    v.visit_object_key(key);
    v.visit_object_value(value);
}

pub fn visit_object_key<V>(v: &mut V, node: &ObjectKey)
where
    V: Visit + ?Sized,
{
    match node {
        ObjectKey::Ident(ident) => v.visit_ident(ident),
        ObjectKey::Expression(expr) => v.visit_expr(expr),
    }
}

pub fn visit_object_value<V>(v: &mut V, node: &ObjectValue)
where
    V: Visit + ?Sized,
{
    v.visit_expr(node.expr());
}

pub fn visit_parenthesis<V>(v: &mut V, node: &Parenthesis)
where
    V: Visit + ?Sized,
{
    v.visit_expr(node.inner());
}

pub fn visit_conditional<V>(v: &mut V, node: &Conditional)
where
    V: Visit + ?Sized,
{
    v.visit_expr(&node.cond_expr);
    v.visit_expr(&node.true_expr);
    v.visit_expr(&node.false_expr);
}

pub fn visit_unary_op<V>(v: &mut V, node: &UnaryOp)
where
    V: Visit + ?Sized,
{
    v.visit_unary_operator(&node.operator);
    v.visit_expr(&node.expr);
}

pub fn visit_binary_op<V>(v: &mut V, node: &BinaryOp)
where
    V: Visit + ?Sized,
{
    v.visit_expr(&node.lhs_expr);
    v.visit_binary_operator(&node.operator);
    v.visit_expr(&node.rhs_expr);
}

pub fn visit_traversal<V>(v: &mut V, node: &Traversal)
where
    V: Visit + ?Sized,
{
    v.visit_expr(&node.expr);
    for operator in &node.operators {
        v.visit_traversal_operator(operator);
    }
}

pub fn visit_traversal_operator<V>(v: &mut V, node: &TraversalOperator)
where
    V: Visit + ?Sized,
{
    match node {
        TraversalOperator::AttrSplat(splat) | TraversalOperator::FullSplat(splat) => {
            v.visit_splat(splat);
        }
        TraversalOperator::GetAttr(ident) => v.visit_ident(ident),
        TraversalOperator::Index(expr) => v.visit_expr(expr),
        TraversalOperator::LegacyIndex(u) => v.visit_u64(u),
    }
}

pub fn visit_func_call<V>(v: &mut V, node: &FuncCall)
where
    V: Visit + ?Sized,
{
    v.visit_ident(&node.ident);
    v.visit_func_args(&node.args);
}

pub fn visit_func_args<V>(v: &mut V, node: &FuncArgs)
where
    V: Visit + ?Sized,
{
    for arg in node.iter() {
        v.visit_expr(arg);
    }
}

pub fn visit_for_expr<V>(v: &mut V, node: &ForExpr)
where
    V: Visit + ?Sized,
{
    v.visit_for_intro(&node.intro);
    if let Some(key_expr) = &node.key_expr {
        v.visit_expr(key_expr);
    }
    v.visit_expr(&node.value_expr);
    if let Some(cond) = &node.cond {
        v.visit_for_cond(cond);
    }
}

pub fn visit_for_intro<V>(v: &mut V, node: &ForIntro)
where
    V: Visit + ?Sized,
{
    if let Some(key_var) = &node.key_var {
        v.visit_ident(key_var);
    }
    v.visit_ident(&node.value_var);
    v.visit_expr(&node.collection_expr);
}

pub fn visit_for_cond<V>(v: &mut V, node: &ForCond)
where
    V: Visit + ?Sized,
{
    v.visit_expr(&node.expr);
}

pub fn visit_string_template<V>(v: &mut V, node: &StringTemplate)
where
    V: Visit + ?Sized,
{
    for element in node.iter() {
        v.visit_element(element);
    }
}

pub fn visit_heredoc_template<V>(v: &mut V, node: &HeredocTemplate)
where
    V: Visit + ?Sized,
{
    v.visit_template(&node.template);
}

pub fn visit_template<V>(v: &mut V, node: &Template)
where
    V: Visit + ?Sized,
{
    for element in node.iter() {
        v.visit_element(element);
    }
}

pub fn visit_element<V>(v: &mut V, node: &Element)
where
    V: Visit + ?Sized,
{
    match node {
        Element::Literal(literal) => v.visit_literal(literal),
        Element::Interpolation(interpolation) => v.visit_interpolation(interpolation),
        Element::Directive(directive) => v.visit_directive(directive),
    }
}

pub fn visit_interpolation<V>(v: &mut V, node: &Interpolation)
where
    V: Visit + ?Sized,
{
    v.visit_expr(&node.expr);
}

pub fn visit_directive<V>(v: &mut V, node: &Directive)
where
    V: Visit + ?Sized,
{
    match node {
        Directive::If(if_directive) => v.visit_if_directive(if_directive),
        Directive::For(for_directive) => v.visit_for_directive(for_directive),
    }
}

pub fn visit_if_directive<V>(v: &mut V, node: &IfDirective)
where
    V: Visit + ?Sized,
{
    v.visit_if_template_expr(&node.if_expr);
    if let Some(else_template_expr) = &node.else_expr {
        v.visit_else_template_expr(else_template_expr);
    }
    v.visit_endif_template_expr(&node.endif_expr);
}

pub fn visit_for_directive<V>(v: &mut V, node: &ForDirective)
where
    V: Visit + ?Sized,
{
    v.visit_for_template_expr(&node.for_expr);
    v.visit_endfor_template_expr(&node.endfor_expr);
}

pub fn visit_if_template_expr<V>(v: &mut V, node: &IfTemplateExpr)
where
    V: Visit + ?Sized,
{
    v.visit_expr(&node.cond_expr);
    v.visit_template(&node.template);
}

pub fn visit_else_template_expr<V>(v: &mut V, node: &ElseTemplateExpr)
where
    V: Visit + ?Sized,
{
    v.visit_template(&node.template);
}

pub fn visit_for_template_expr<V>(v: &mut V, node: &ForTemplateExpr)
where
    V: Visit + ?Sized,
{
    if let Some(key_var) = &node.key_var {
        v.visit_ident(key_var);
    }
    v.visit_ident(&node.value_var);
    v.visit_template(&node.template);
}
