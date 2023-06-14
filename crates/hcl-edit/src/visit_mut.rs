//! Mutable HCL language item traversal.
//!
//! Each method of the [`VisitMut`] trait is a hook that can be overridden to customize the
//! behavior when mutating the corresponding type of language item. By default, every method
//! recursively visits the substructure of the AST by invoking the right visitor method of each of
//! its fields.
//!
//! The API is modeled after
//! [`syn::visit_mut`](https://docs.rs/syn/latest/syn/visit_mut/index.html). For an alternative
//! that works on shared borrows, see [`hcl_edit::visit`](crate::visit).
//!
//! # Examples
//!
//! Namespace all referenced variables with `var.`:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use hcl_edit::expr::{Expression, Traversal, TraversalOperator};
//! use hcl_edit::prelude::*;
//! use hcl_edit::structure::Body;
//! use hcl_edit::visit_mut::{visit_expr_mut, VisitMut};
//! use hcl_edit::{Decorated, Ident};
//! use std::str::FromStr;
//!
//! struct VariableNamespacer {
//!     namespace: Decorated<Ident>,
//! }
//!
//! impl VisitMut for VariableNamespacer {
//!     fn visit_expr_mut(&mut self, expr: & mut Expression) {
//!         if let Expression::Variable(var) = expr {
//!             // Remove the decor and apply it to the new expression.
//!             let decor = std::mem::take(var.decor_mut());
//!
//!             let namespace = Expression::Variable(self.namespace.clone());
//!             let operators = vec![Decorated::new(TraversalOperator::GetAttr(var.clone()))];
//!             let traversal = Traversal::new(namespace, operators);
//!
//!             *expr = Expression::from(traversal).decorated(decor);
//!         } else {
//!             // Recurse further down the AST.
//!             visit_expr_mut(self, expr);
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
//! let mut body = input.parse::<Body>()?;
//!
//! let mut visitor = VariableNamespacer {
//!     namespace: Decorated::new(Ident::new("var")),
//! };
//!
//! visitor.visit_body_mut(&mut body);
//!
//! let expected = r#"
//!     // A service definition.
//!     service {
//!         fullname        = "${var.namespace}/${var.name}"
//!         health_endpoint = "${var.base_url}/health"
//!     }
//! "#;
//!
//! assert_eq!(body.to_string(), expected);
//! #   Ok(())
//! # }
//! ```

#![allow(missing_docs)]

use crate::expr::{
    Array, BinaryOp, BinaryOperator, Conditional, Expression, ForCond, ForExpr, ForIntro, FuncArgs,
    FuncCall, Null, Object, ObjectKeyMut, ObjectValue, Parenthesis, Splat, Traversal,
    TraversalOperator, UnaryOp, UnaryOperator,
};
use crate::structure::{AttributeMut, Block, BlockLabel, Body, StructureMut};
use crate::template::{
    Directive, Element, ElseTemplateExpr, EndforTemplateExpr, EndifTemplateExpr, EscapedLiteral,
    ForDirective, ForTemplateExpr, HeredocTemplate, IfDirective, IfTemplateExpr, Interpolation,
    StringTemplate, Template,
};
use crate::{Decorated, Formatted, Ident, Number, Spanned};

macro_rules! empty_visit_mut_methods {
    ($($name: ident => $t: ty),+ $(,)?) => {
        $(
            fn $name(&mut self, node: &mut $t) {
                let _ = node;
            }
        )*
    };
}

macro_rules! visit_mut_methods {
    ($($name: ident => $t: ty),+ $(,)?) => {
        $(
            fn $name(&mut self, node: &mut $t) {
                $name(self, node);
            }
        )*
    };
}

/// Traversal to walk a mutable borrow of an HCL language item.
///
/// See the [module documentation](crate::visit_mut) for details.
pub trait VisitMut {
    empty_visit_mut_methods! {
        visit_ident_mut => Decorated<Ident>,
        visit_null_mut => Decorated<Null>,
        visit_bool_mut => Decorated<bool>,
        visit_u64_mut => Decorated<u64>,
        visit_number_mut => Formatted<Number>,
        visit_string_mut => Decorated<String>,
        visit_splat_mut => Decorated<Splat>,
        visit_literal_mut => Spanned<String>,
        visit_escaped_literal_mut => Spanned<EscapedLiteral>,
        visit_unary_operator_mut => Spanned<UnaryOperator>,
        visit_binary_operator_mut => Spanned<BinaryOperator>,
        visit_endif_template_expr_mut => EndifTemplateExpr,
        visit_endfor_template_expr_mut => EndforTemplateExpr,
    }

    visit_mut_methods! {
        visit_body_mut => Body,
        visit_block_mut => Block,
        visit_block_label_mut => BlockLabel,
        visit_expr_mut => Expression,
        visit_array_mut => Array,
        visit_object_mut => Object,
        visit_object_value_mut => ObjectValue,
        visit_parenthesis_mut => Parenthesis,
        visit_conditional_mut => Conditional,
        visit_unary_op_mut => UnaryOp,
        visit_binary_op_mut => BinaryOp,
        visit_traversal_mut => Traversal,
        visit_traversal_operator_mut => TraversalOperator,
        visit_func_call_mut => FuncCall,
        visit_func_args_mut => FuncArgs,
        visit_for_expr_mut => ForExpr,
        visit_for_intro_mut => ForIntro,
        visit_for_cond_mut => ForCond,
        visit_string_template_mut => StringTemplate,
        visit_heredoc_template_mut => HeredocTemplate,
        visit_template_mut => Template,
        visit_element_mut => Element,
        visit_interpolation_mut => Interpolation,
        visit_directive_mut => Directive,
        visit_if_directive_mut => IfDirective,
        visit_for_directive_mut => ForDirective,
        visit_if_template_expr_mut => IfTemplateExpr,
        visit_else_template_expr_mut => ElseTemplateExpr,
        visit_for_template_expr_mut => ForTemplateExpr,
    }

    fn visit_structure_mut(&mut self, node: StructureMut) {
        visit_structure_mut(self, node);
    }

    fn visit_attr_mut(&mut self, node: AttributeMut) {
        visit_attr_mut(self, node);
    }

    fn visit_object_key_mut(&mut self, node: ObjectKeyMut) {
        let _ = node;
    }

    fn visit_object_item_mut(&mut self, key: ObjectKeyMut, value: &mut ObjectValue) {
        visit_object_item_mut(self, key, value);
    }
}

pub fn visit_body_mut<V>(v: &mut V, node: &mut Body)
where
    V: VisitMut + ?Sized,
{
    for structure in node.iter_mut() {
        v.visit_structure_mut(structure);
    }
}

pub fn visit_structure_mut<V>(v: &mut V, mut node: StructureMut)
where
    V: VisitMut + ?Sized,
{
    if let Some(attr) = node.as_attribute_mut() {
        v.visit_attr_mut(attr);
    } else if let Some(block) = node.as_block_mut() {
        v.visit_block_mut(block);
    }
}

pub fn visit_attr_mut<V>(v: &mut V, mut node: AttributeMut)
where
    V: VisitMut + ?Sized,
{
    v.visit_expr_mut(node.value_mut());
}

pub fn visit_block_mut<V>(v: &mut V, node: &mut Block)
where
    V: VisitMut + ?Sized,
{
    v.visit_ident_mut(&mut node.ident);
    for label in &mut node.labels {
        v.visit_block_label_mut(label);
    }
    v.visit_body_mut(&mut node.body);
}

pub fn visit_block_label_mut<V>(v: &mut V, node: &mut BlockLabel)
where
    V: VisitMut + ?Sized,
{
    match node {
        BlockLabel::String(string) => v.visit_string_mut(string),
        BlockLabel::Ident(ident) => v.visit_ident_mut(ident),
    }
}

pub fn visit_expr_mut<V>(v: &mut V, node: &mut Expression)
where
    V: VisitMut + ?Sized,
{
    match node {
        Expression::Null(null) => v.visit_null_mut(null),
        Expression::Bool(b) => v.visit_bool_mut(b),
        Expression::Number(number) => v.visit_number_mut(number),
        Expression::String(string) => v.visit_string_mut(string),
        Expression::Array(array) => v.visit_array_mut(array),
        Expression::Object(object) => v.visit_object_mut(object),
        Expression::Template(template) => v.visit_string_template_mut(template),
        Expression::HeredocTemplate(template) => v.visit_heredoc_template_mut(template),
        Expression::Parenthesis(parens) => v.visit_parenthesis_mut(parens),
        Expression::Variable(var) => v.visit_ident_mut(var),
        Expression::ForExpr(for_expr) => v.visit_for_expr_mut(for_expr),
        Expression::Conditional(conditional) => v.visit_conditional_mut(conditional),
        Expression::FuncCall(func_call) => v.visit_func_call_mut(func_call),
        Expression::UnaryOp(unary_op) => v.visit_unary_op_mut(unary_op),
        Expression::BinaryOp(binary_op) => v.visit_binary_op_mut(binary_op),
        Expression::Traversal(traversal) => v.visit_traversal_mut(traversal),
    }
}

pub fn visit_array_mut<V>(v: &mut V, node: &mut Array)
where
    V: VisitMut + ?Sized,
{
    for expr in node.iter_mut() {
        v.visit_expr_mut(expr);
    }
}

pub fn visit_object_mut<V>(v: &mut V, node: &mut Object)
where
    V: VisitMut + ?Sized,
{
    for (key, value) in node.iter_mut() {
        v.visit_object_item_mut(key, value);
    }
}

pub fn visit_object_item_mut<V>(v: &mut V, key: ObjectKeyMut, value: &mut ObjectValue)
where
    V: VisitMut + ?Sized,
{
    v.visit_object_key_mut(key);
    v.visit_object_value_mut(value);
}

pub fn visit_object_value_mut<V>(v: &mut V, node: &mut ObjectValue)
where
    V: VisitMut + ?Sized,
{
    v.visit_expr_mut(node.expr_mut());
}

pub fn visit_parenthesis_mut<V>(v: &mut V, node: &mut Parenthesis)
where
    V: VisitMut + ?Sized,
{
    v.visit_expr_mut(node.inner_mut());
}

pub fn visit_conditional_mut<V>(v: &mut V, node: &mut Conditional)
where
    V: VisitMut + ?Sized,
{
    v.visit_expr_mut(&mut node.cond_expr);
    v.visit_expr_mut(&mut node.true_expr);
    v.visit_expr_mut(&mut node.false_expr);
}

pub fn visit_unary_op_mut<V>(v: &mut V, node: &mut UnaryOp)
where
    V: VisitMut + ?Sized,
{
    v.visit_unary_operator_mut(&mut node.operator);
    v.visit_expr_mut(&mut node.expr);
}

pub fn visit_binary_op_mut<V>(v: &mut V, node: &mut BinaryOp)
where
    V: VisitMut + ?Sized,
{
    v.visit_expr_mut(&mut node.lhs_expr);
    v.visit_binary_operator_mut(&mut node.operator);
    v.visit_expr_mut(&mut node.rhs_expr);
}

pub fn visit_traversal_mut<V>(v: &mut V, node: &mut Traversal)
where
    V: VisitMut + ?Sized,
{
    v.visit_expr_mut(&mut node.expr);
    for operator in &mut node.operators {
        v.visit_traversal_operator_mut(operator);
    }
}

pub fn visit_traversal_operator_mut<V>(v: &mut V, node: &mut TraversalOperator)
where
    V: VisitMut + ?Sized,
{
    match node {
        TraversalOperator::AttrSplat(splat) | TraversalOperator::FullSplat(splat) => {
            v.visit_splat_mut(splat);
        }
        TraversalOperator::GetAttr(ident) => v.visit_ident_mut(ident),
        TraversalOperator::Index(expr) => v.visit_expr_mut(expr),
        TraversalOperator::LegacyIndex(u) => v.visit_u64_mut(u),
    }
}

pub fn visit_func_call_mut<V>(v: &mut V, node: &mut FuncCall)
where
    V: VisitMut + ?Sized,
{
    v.visit_ident_mut(&mut node.ident);
    v.visit_func_args_mut(&mut node.args);
}

pub fn visit_func_args_mut<V>(v: &mut V, node: &mut FuncArgs)
where
    V: VisitMut + ?Sized,
{
    for arg in node.iter_mut() {
        v.visit_expr_mut(arg);
    }
}

pub fn visit_for_expr_mut<V>(v: &mut V, node: &mut ForExpr)
where
    V: VisitMut + ?Sized,
{
    v.visit_for_intro_mut(&mut node.intro);
    if let Some(key_expr) = &mut node.key_expr {
        v.visit_expr_mut(key_expr);
    }
    v.visit_expr_mut(&mut node.value_expr);
    if let Some(cond) = &mut node.cond {
        v.visit_for_cond_mut(cond);
    }
}

pub fn visit_for_intro_mut<V>(v: &mut V, node: &mut ForIntro)
where
    V: VisitMut + ?Sized,
{
    if let Some(key_var) = &mut node.key_var {
        v.visit_ident_mut(key_var);
    }
    v.visit_ident_mut(&mut node.value_var);
    v.visit_expr_mut(&mut node.collection_expr);
}

pub fn visit_for_cond_mut<V>(v: &mut V, node: &mut ForCond)
where
    V: VisitMut + ?Sized,
{
    v.visit_expr_mut(&mut node.expr);
}

pub fn visit_string_template_mut<V>(v: &mut V, node: &mut StringTemplate)
where
    V: VisitMut + ?Sized,
{
    for element in node.iter_mut() {
        v.visit_element_mut(element);
    }
}

pub fn visit_heredoc_template_mut<V>(v: &mut V, node: &mut HeredocTemplate)
where
    V: VisitMut + ?Sized,
{
    v.visit_template_mut(&mut node.template);
}

pub fn visit_template_mut<V>(v: &mut V, node: &mut Template)
where
    V: VisitMut + ?Sized,
{
    for element in node.iter_mut() {
        v.visit_element_mut(element);
    }
}

pub fn visit_element_mut<V>(v: &mut V, node: &mut Element)
where
    V: VisitMut + ?Sized,
{
    match node {
        Element::Literal(literal) => v.visit_literal_mut(literal),
        Element::EscapedLiteral(literal) => v.visit_escaped_literal_mut(literal),
        Element::Interpolation(interpolation) => v.visit_interpolation_mut(interpolation),
        Element::Directive(directive) => v.visit_directive_mut(directive),
    }
}

pub fn visit_interpolation_mut<V>(v: &mut V, node: &mut Interpolation)
where
    V: VisitMut + ?Sized,
{
    v.visit_expr_mut(&mut node.expr);
}

pub fn visit_directive_mut<V>(v: &mut V, node: &mut Directive)
where
    V: VisitMut + ?Sized,
{
    match node {
        Directive::If(if_directive) => v.visit_if_directive_mut(if_directive),
        Directive::For(for_directive) => v.visit_for_directive_mut(for_directive),
    }
}

pub fn visit_if_directive_mut<V>(v: &mut V, node: &mut IfDirective)
where
    V: VisitMut + ?Sized,
{
    v.visit_if_template_expr_mut(&mut node.if_expr);
    if let Some(else_template_expr) = &mut node.else_expr {
        v.visit_else_template_expr_mut(else_template_expr);
    }
    v.visit_endif_template_expr_mut(&mut node.endif_expr);
}

pub fn visit_for_directive_mut<V>(v: &mut V, node: &mut ForDirective)
where
    V: VisitMut + ?Sized,
{
    v.visit_for_template_expr_mut(&mut node.for_expr);
    v.visit_endfor_template_expr_mut(&mut node.endfor_expr);
}

pub fn visit_if_template_expr_mut<V>(v: &mut V, node: &mut IfTemplateExpr)
where
    V: VisitMut + ?Sized,
{
    v.visit_expr_mut(&mut node.cond_expr);
    v.visit_template_mut(&mut node.template);
}

pub fn visit_else_template_expr_mut<V>(v: &mut V, node: &mut ElseTemplateExpr)
where
    V: VisitMut + ?Sized,
{
    v.visit_template_mut(&mut node.template);
}

pub fn visit_for_template_expr_mut<V>(v: &mut V, node: &mut ForTemplateExpr)
where
    V: VisitMut + ?Sized,
{
    if let Some(key_var) = &mut node.key_var {
        v.visit_ident_mut(key_var);
    }
    v.visit_ident_mut(&mut node.value_var);
    v.visit_template_mut(&mut node.template);
}
