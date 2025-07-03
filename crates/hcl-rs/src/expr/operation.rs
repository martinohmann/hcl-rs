use super::Expression;
use serde::Deserialize;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::expr::{BinaryOperator, UnaryOperator};

/// Operations apply a particular operator to either one or two expression terms.
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Operation {
    /// Represents an operation that applies an operator to a single expression.
    Unary(UnaryOp),
    /// Represents an operation that applies an operator to two expressions.
    Binary(BinaryOp),
}

impl From<UnaryOp> for Operation {
    fn from(op: UnaryOp) -> Self {
        Operation::Unary(op)
    }
}

impl From<BinaryOp> for Operation {
    fn from(op: BinaryOp) -> Self {
        Operation::Binary(op)
    }
}

/// An operation that applies an operator to one expression.
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct UnaryOp {
    /// The unary operator to use on the expression.
    pub operator: UnaryOperator,
    /// An expression that supports evaluation with the unary operator.
    pub expr: Expression,
}

impl UnaryOp {
    /// Creates a new `UnaryOp` from an operator and an expression.
    pub fn new<T>(operator: UnaryOperator, expr: T) -> UnaryOp
    where
        T: Into<Expression>,
    {
        UnaryOp {
            operator,
            expr: expr.into(),
        }
    }
}

/// An operation that applies an operator to two expressions.
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct BinaryOp {
    /// The expression on the left-hand-side of the operation.
    pub lhs_expr: Expression,
    /// The binary operator to use on the expressions.
    pub operator: BinaryOperator,
    /// The expression on the right-hand-side of the operation.
    pub rhs_expr: Expression,
}

impl BinaryOp {
    /// Creates a new `BinaryOp` from two expressions and an operator.
    pub fn new<L, R>(lhs_expr: L, operator: BinaryOperator, rhs_expr: R) -> BinaryOp
    where
        L: Into<Expression>,
        R: Into<Expression>,
    {
        BinaryOp {
            lhs_expr: lhs_expr.into(),
            operator,
            rhs_expr: rhs_expr.into(),
        }
    }
}
