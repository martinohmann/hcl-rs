use crate::expr::Expression;
use crate::{Decor, Spanned};
use std::ops::Range;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::expr::{BinaryOperator, UnaryOperator};

/// An operation that applies an operator to one expression.
#[derive(Debug, Clone, Eq)]
pub struct UnaryOp {
    /// The unary operator to use on the expression.
    pub operator: Spanned<UnaryOperator>,
    /// An expression that supports evaluation with the unary operator.
    pub expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl UnaryOp {
    /// Creates a new `UnaryOp` from an operator and an expression.
    pub fn new(
        operator: impl Into<Spanned<UnaryOperator>>,
        expr: impl Into<Expression>,
    ) -> UnaryOp {
        UnaryOp {
            operator: operator.into(),
            expr: expr.into(),
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);
    }
}

impl PartialEq for UnaryOp {
    fn eq(&self, other: &Self) -> bool {
        self.operator == other.operator && self.expr == other.expr
    }
}

/// An operation that applies an operator to two expressions.
#[derive(Debug, Clone, Eq)]
pub struct BinaryOp {
    /// The expression on the left-hand-side of the operation.
    pub lhs_expr: Expression,
    /// The binary operator to use on the expressions.
    pub operator: Spanned<BinaryOperator>,
    /// The expression on the right-hand-side of the operation.
    pub rhs_expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl BinaryOp {
    /// Creates a new `BinaryOp` from two expressions and an operator.
    pub fn new(
        lhs_expr: impl Into<Expression>,
        operator: impl Into<Spanned<BinaryOperator>>,
        rhs_expr: impl Into<Expression>,
    ) -> BinaryOp {
        BinaryOp {
            lhs_expr: lhs_expr.into(),
            operator: operator.into(),
            rhs_expr: rhs_expr.into(),
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.lhs_expr.despan(input);
        self.rhs_expr.despan(input);
    }
}

impl PartialEq for BinaryOp {
    fn eq(&self, other: &Self) -> bool {
        self.lhs_expr == other.lhs_expr
            && self.operator == other.operator
            && self.rhs_expr == other.rhs_expr
    }
}

decorate_impl!(UnaryOp, BinaryOp);
span_impl!(UnaryOp, BinaryOp);
