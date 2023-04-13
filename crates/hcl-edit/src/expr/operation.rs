#![allow(missing_docs)]

use crate::expr::Expression;
use crate::repr::{Decor, Decorate, SetSpan, Span, Spanned};
use std::ops::Range;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::expr::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone, Eq)]
pub struct UnaryOp {
    pub operator: Spanned<UnaryOperator>,
    pub expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl UnaryOp {
    pub fn new(operator: Spanned<UnaryOperator>, expr: Expression) -> UnaryOp {
        UnaryOp {
            operator,
            expr,
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

#[derive(Debug, Clone, Eq)]
pub struct BinaryOp {
    pub lhs_expr: Expression,
    pub operator: Spanned<BinaryOperator>,
    pub rhs_expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl BinaryOp {
    pub fn new(
        lhs_expr: Expression,
        operator: Spanned<BinaryOperator>,
        rhs_expr: Expression,
    ) -> BinaryOp {
        BinaryOp {
            lhs_expr,
            operator,
            rhs_expr,
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
