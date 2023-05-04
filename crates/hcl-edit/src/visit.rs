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
//! struct VariableNameVisitor<'a> {
//!     variable_names: HashSet<&'a str>,
//! }
//!
//! impl<'ast> Visit<'ast> for VariableNameVisitor<'ast> {
//!     fn visit_expr(&mut self, expr: &'ast Expression) {
//!         if let Expression::Variable(var) = expr {
//!             self.variable_names.insert(var.as_str());
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
//! let expected = HashSet::from(["namespace", "name", "base_url"]);
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
use crate::repr::{Decorated, Formatted, Spanned};
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use crate::template::{
    Directive, Element, ElseTemplateExpr, EndforTemplateExpr, EndifTemplateExpr, ForDirective,
    ForTemplateExpr, HeredocTemplate, IfDirective, IfTemplateExpr, Interpolation, StringTemplate,
    Template,
};
use crate::{Ident, Number};

macro_rules! empty_visit_methods {
    ($($name: ident => $t: ty),+ $(,)?) => {
        $(
            fn $name(&mut self, node: &'ast $t) {
                let _ = node;
            }
        )*
    };
}

macro_rules! visit_methods {
    ($($name: ident => $t: ty),+ $(,)?) => {
        $(
            fn $name(&mut self, node: &'ast $t) {
                $name(self, node);
            }
        )*
    };
}

/// Traversal to walk a shared borrow of an HCL language item.
///
/// See the [module documentation](crate::visit) for details.
pub trait Visit<'ast> {
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

    fn visit_object_item(&mut self, key: &'ast ObjectKey, value: &'ast ObjectValue) {
        visit_object_item(self, key, value);
    }
}

pub fn visit_body<'ast, V>(v: &mut V, node: &'ast Body)
where
    V: Visit<'ast> + ?Sized,
{
    for structure in node.iter() {
        v.visit_structure(structure);
    }
}

pub fn visit_structure<'ast, V>(v: &mut V, node: &'ast Structure)
where
    V: Visit<'ast> + ?Sized,
{
    match node {
        Structure::Attribute(attr) => v.visit_attr(attr),
        Structure::Block(block) => v.visit_block(block),
    }
}

pub fn visit_attr<'ast, V>(v: &mut V, node: &'ast Attribute)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_ident(&node.key);
    v.visit_expr(&node.value);
}

pub fn visit_block<'ast, V>(v: &mut V, node: &'ast Block)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_ident(&node.ident);
    for label in &node.labels {
        v.visit_block_label(label);
    }
    v.visit_body(&node.body);
}

pub fn visit_block_label<'ast, V>(v: &mut V, node: &'ast BlockLabel)
where
    V: Visit<'ast> + ?Sized,
{
    match node {
        BlockLabel::String(string) => v.visit_string(string),
        BlockLabel::Ident(ident) => v.visit_ident(ident),
    }
}

pub fn visit_expr<'ast, V>(v: &mut V, node: &'ast Expression)
where
    V: Visit<'ast> + ?Sized,
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

pub fn visit_array<'ast, V>(v: &mut V, node: &'ast Array)
where
    V: Visit<'ast> + ?Sized,
{
    for expr in node.iter() {
        v.visit_expr(expr);
    }
}

pub fn visit_object<'ast, V>(v: &mut V, node: &'ast Object)
where
    V: Visit<'ast> + ?Sized,
{
    for (key, value) in node.iter() {
        v.visit_object_item(key, value);
    }
}

pub fn visit_object_item<'ast, V>(v: &mut V, key: &'ast ObjectKey, value: &'ast ObjectValue)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_object_key(key);
    v.visit_object_value(value);
}

pub fn visit_object_key<'ast, V>(v: &mut V, node: &'ast ObjectKey)
where
    V: Visit<'ast> + ?Sized,
{
    match node {
        ObjectKey::Ident(ident) => v.visit_ident(ident),
        ObjectKey::Expression(expr) => v.visit_expr(expr),
    }
}

pub fn visit_object_value<'ast, V>(v: &mut V, node: &'ast ObjectValue)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(node.expr());
}

pub fn visit_parenthesis<'ast, V>(v: &mut V, node: &'ast Parenthesis)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(node.inner());
}

pub fn visit_conditional<'ast, V>(v: &mut V, node: &'ast Conditional)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.cond_expr);
    v.visit_expr(&node.true_expr);
    v.visit_expr(&node.false_expr);
}

pub fn visit_unary_op<'ast, V>(v: &mut V, node: &'ast UnaryOp)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_unary_operator(&node.operator);
    v.visit_expr(&node.expr);
}

pub fn visit_binary_op<'ast, V>(v: &mut V, node: &'ast BinaryOp)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.lhs_expr);
    v.visit_binary_operator(&node.operator);
    v.visit_expr(&node.rhs_expr);
}

pub fn visit_traversal<'ast, V>(v: &mut V, node: &'ast Traversal)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.expr);
    for operator in &node.operators {
        v.visit_traversal_operator(operator);
    }
}

pub fn visit_traversal_operator<'ast, V>(v: &mut V, node: &'ast TraversalOperator)
where
    V: Visit<'ast> + ?Sized,
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

pub fn visit_func_call<'ast, V>(v: &mut V, node: &'ast FuncCall)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_ident(&node.ident);
    v.visit_func_args(&node.args);
}

pub fn visit_func_args<'ast, V>(v: &mut V, node: &'ast FuncArgs)
where
    V: Visit<'ast> + ?Sized,
{
    for arg in node.iter() {
        v.visit_expr(arg);
    }
}

pub fn visit_for_expr<'ast, V>(v: &mut V, node: &'ast ForExpr)
where
    V: Visit<'ast> + ?Sized,
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

pub fn visit_for_intro<'ast, V>(v: &mut V, node: &'ast ForIntro)
where
    V: Visit<'ast> + ?Sized,
{
    if let Some(key_var) = &node.key_var {
        v.visit_ident(key_var);
    }
    v.visit_ident(&node.value_var);
    v.visit_expr(&node.collection_expr);
}

pub fn visit_for_cond<'ast, V>(v: &mut V, node: &'ast ForCond)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.expr);
}

pub fn visit_string_template<'ast, V>(v: &mut V, node: &'ast StringTemplate)
where
    V: Visit<'ast> + ?Sized,
{
    for element in node.iter() {
        v.visit_element(element);
    }
}

pub fn visit_heredoc_template<'ast, V>(v: &mut V, node: &'ast HeredocTemplate)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_template(&node.template);
}

pub fn visit_template<'ast, V>(v: &mut V, node: &'ast Template)
where
    V: Visit<'ast> + ?Sized,
{
    for element in node.iter() {
        v.visit_element(element);
    }
}

pub fn visit_element<'ast, V>(v: &mut V, node: &'ast Element)
where
    V: Visit<'ast> + ?Sized,
{
    match node {
        Element::Literal(literal) => v.visit_literal(literal),
        Element::Interpolation(interpolation) => v.visit_interpolation(interpolation),
        Element::Directive(directive) => v.visit_directive(directive),
    }
}

pub fn visit_interpolation<'ast, V>(v: &mut V, node: &'ast Interpolation)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.expr);
}

pub fn visit_directive<'ast, V>(v: &mut V, node: &'ast Directive)
where
    V: Visit<'ast> + ?Sized,
{
    match node {
        Directive::If(if_directive) => v.visit_if_directive(if_directive),
        Directive::For(for_directive) => v.visit_for_directive(for_directive),
    }
}

pub fn visit_if_directive<'ast, V>(v: &mut V, node: &'ast IfDirective)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_if_template_expr(&node.if_expr);
    if let Some(else_template_expr) = &node.else_expr {
        v.visit_else_template_expr(else_template_expr);
    }
    v.visit_endif_template_expr(&node.endif_expr);
}

pub fn visit_for_directive<'ast, V>(v: &mut V, node: &'ast ForDirective)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_for_template_expr(&node.for_expr);
    v.visit_endfor_template_expr(&node.endfor_expr);
}

pub fn visit_if_template_expr<'ast, V>(v: &mut V, node: &'ast IfTemplateExpr)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_expr(&node.cond_expr);
    v.visit_template(&node.template);
}

pub fn visit_else_template_expr<'ast, V>(v: &mut V, node: &'ast ElseTemplateExpr)
where
    V: Visit<'ast> + ?Sized,
{
    v.visit_template(&node.template);
}

pub fn visit_for_template_expr<'ast, V>(v: &mut V, node: &'ast ForTemplateExpr)
where
    V: Visit<'ast> + ?Sized,
{
    if let Some(key_var) = &node.key_var {
        v.visit_ident(key_var);
    }
    v.visit_ident(&node.value_var);
    v.visit_template(&node.template);
}
