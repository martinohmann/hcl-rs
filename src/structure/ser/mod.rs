//! Serializer impls for HCL structure types.

mod attribute;
mod block;
pub(crate) mod body;
mod conditional;
mod expression;
mod for_expr;
mod func_call;
mod operation;
mod structure;
mod template_expr;
#[cfg(test)]
mod tests;
mod traversal;

pub use self::expression::to_expression;
