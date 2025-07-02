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

    // Normalize binary operation following operator precedence rules.
    //
    // The result can be evaluated from left to right without checking operator precendence.
    pub(crate) fn normalize(self) -> BinaryOp {
        let lhs_denormal = self.lhs_expr.as_binary_op().map_or(false, |lhs| {
            lhs.operator.precedence() < self.operator.precedence()
        });

        let rhs_denormal = self.rhs_expr.as_binary_op().map_or(false, |rhs| {
            rhs.operator.precedence() < self.operator.precedence()
        });

        if !lhs_denormal && !rhs_denormal {
            // Fast path: nothing to normalize.
            return self;
        }

        let operator = self.operator;

        match (self.lhs_expr, self.rhs_expr) {
            (Expression::BinaryOp(lhs), Expression::BinaryOp(rhs)) => {
                normalize_both(lhs.normalize(), operator, rhs.normalize())
            }
            (Expression::BinaryOp(lhs), rhs) => normalize_lhs(lhs.normalize(), operator, rhs),
            (lhs, Expression::BinaryOp(rhs)) => normalize_rhs(lhs, operator, rhs.normalize()),
            (_, _) => unreachable!(),
        }
    }
}

fn normalize_both(lhs: BinaryOp, operator: Spanned<BinaryOperator>, rhs: BinaryOp) -> BinaryOp {
    if lhs.operator.precedence() < operator.precedence() {
        // BinaryOp(BinaryOp(lhs.lhs_expr + lhs.rhs_expr) * BinaryOp(rhs.lhs_expr - rhs.rhs_expr))
        //
        // => BinaryOp(lhs.lhs_expr + BinaryOp(BinaryOp(lhs.rhs_expr * rhs.lhs_expr) - rhs.rhs_expr))
        BinaryOp::new(
            lhs.lhs_expr,
            lhs.operator,
            normalize_rhs(lhs.rhs_expr, operator, rhs),
        )
    } else if rhs.operator.precedence() < operator.precedence() {
        // BinaryOp(BinaryOp(lhs.lhs_expr / lhs.rhs_expr) * BinaryOp(rhs.lhs_expr - rhs.rhs_expr))
        //
        // => BinaryOp(BinaryOp(BinaryOp(lhs.lhs_expr / lhs.rhs_expr) * rhs.lhs_expr) - rhs.rhs_expr)
        BinaryOp::new(
            normalize_lhs(lhs, operator, rhs.lhs_expr),
            rhs.operator,
            rhs.rhs_expr,
        )
    } else {
        // Nothing to normalize.
        BinaryOp::new(lhs, operator, rhs)
    }
}

fn normalize_lhs(
    lhs: BinaryOp,
    operator: Spanned<BinaryOperator>,
    rhs_expr: Expression,
) -> BinaryOp {
    if lhs.operator.precedence() < operator.precedence() {
        // BinaryOp(BinaryOp(lhs.lhs_expr + lhs.rhs_expr) / rhs_expr)
        //
        // => BinaryOp(lhs.lhs_expr + BinaryOp(lhs.rhs_expr / rhs_expr))
        BinaryOp::new(
            lhs.lhs_expr,
            lhs.operator,
            BinaryOp::new(lhs.rhs_expr, operator, rhs_expr),
        )
    } else {
        // Nothing to normalize.
        BinaryOp::new(lhs, operator, rhs_expr)
    }
}

fn normalize_rhs(
    lhs_expr: Expression,
    operator: Spanned<BinaryOperator>,
    rhs: BinaryOp,
) -> BinaryOp {
    if rhs.operator.precedence() < operator.precedence() {
        // BinaryOp(lhs_expr / BinaryOp(rhs.lhs_expr + rhs.rhs_expr))
        //
        // => BinaryOp(BinaryOp(lhs_expr / rhs.lhs_expr) + rhs.rhs_expr)
        BinaryOp::new(
            BinaryOp::new(lhs_expr, operator, rhs.lhs_expr),
            rhs.operator,
            rhs.rhs_expr,
        )
    } else {
        // Nothing to normalize.
        BinaryOp::new(lhs_expr, operator, rhs)
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
