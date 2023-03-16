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

    // Normalize binary operation following operator precedence rules.
    //
    // The result can be evaluated from left to right without checking operator precendence.
    pub(crate) fn normalize(self) -> BinaryOp {
        use Operand::{BinOp, Expr};

        // We only care whether the operand is another binary operation or not. Any other
        // expression (including unary oparations) is treated the same way and does not require
        // special precedence rules.
        enum Operand {
            BinOp(BinaryOp),
            Expr(Expression),
        }

        impl From<Expression> for Operand {
            fn from(expr: Expression) -> Self {
                match expr {
                    Expression::Operation(operation) => match *operation {
                        Operation::Binary(binary) => Operand::BinOp(binary),
                        unary => Operand::Expr(Expression::from(unary)),
                    },
                    expr => Operand::Expr(expr),
                }
            }
        }

        let lhs = Operand::from(self.lhs_expr);
        let operator = self.operator;
        let rhs = Operand::from(self.rhs_expr);

        match (lhs, rhs) {
            (BinOp(lhs), BinOp(rhs)) => normalize_both(lhs.normalize(), operator, rhs.normalize()),
            (BinOp(lhs), Expr(rhs)) => normalize_lhs(lhs.normalize(), operator, rhs),
            (Expr(lhs), BinOp(rhs)) => normalize_rhs(lhs, operator, rhs.normalize()),
            (Expr(lhs), Expr(rhs)) => BinaryOp::new(lhs, operator, rhs),
        }
    }
}

fn normalize_both(lhs: BinaryOp, operator: BinaryOperator, rhs: BinaryOp) -> BinaryOp {
    if lhs.operator.precedence() < operator.precedence() {
        // BinaryOp(BinaryOp(lhs.lhs_expr + lhs.rhs_expr) * BinaryOp(rhs.lhs_expr - rhs.rhs_expr))
        //
        // => BinaryOp(lhs.lhs_expr + BinaryOp(BinaryOp(lhs.rhs_expr * rhs.lhs_expr) - rhs.rhs_expr))
        BinaryOp::new(
            lhs.lhs_expr,
            lhs.operator,
            Operation::Binary(normalize_rhs(lhs.rhs_expr, operator, rhs)),
        )
    } else if rhs.operator.precedence() < operator.precedence() {
        // BinaryOp(BinaryOp(lhs.lhs_expr / lhs.rhs_expr) * BinaryOp(rhs.lhs_expr - rhs.rhs_expr))
        //
        // => BinaryOp(BinaryOp(BinaryOp(lhs.lhs_expr / lhs.rhs_expr) * rhs.lhs_expr) - rhs.rhs_expr)
        BinaryOp::new(
            Operation::Binary(normalize_lhs(lhs, operator, rhs.lhs_expr)),
            rhs.operator,
            rhs.rhs_expr,
        )
    } else {
        // Nothing to normalize.
        BinaryOp::new(Operation::Binary(lhs), operator, Operation::Binary(rhs))
    }
}

fn normalize_lhs(lhs: BinaryOp, operator: BinaryOperator, rhs_expr: Expression) -> BinaryOp {
    if lhs.operator.precedence() < operator.precedence() {
        // BinaryOp(BinaryOp(lhs.lhs_expr + lhs.rhs_expr) / rhs_expr)
        //
        // => BinaryOp(lhs.lhs_expr + BinaryOp(lhs.rhs_expr / rhs_expr))
        BinaryOp::new(
            lhs.lhs_expr,
            lhs.operator,
            Operation::Binary(BinaryOp::new(lhs.rhs_expr, operator, rhs_expr)),
        )
    } else {
        // Nothing to normalize.
        BinaryOp::new(Operation::Binary(lhs), operator, rhs_expr)
    }
}

fn normalize_rhs(lhs_expr: Expression, operator: BinaryOperator, rhs: BinaryOp) -> BinaryOp {
    if rhs.operator.precedence() < operator.precedence() {
        // BinaryOp(lhs_expr / BinaryOp(rhs.lhs_expr + rhs.rhs_expr))
        //
        // => BinaryOp(BinaryOp(lhs_expr / rhs.lhs_expr) + rhs.rhs_expr)
        BinaryOp::new(
            Operation::Binary(BinaryOp::new(lhs_expr, operator, rhs.lhs_expr)),
            rhs.operator,
            rhs.rhs_expr,
        )
    } else {
        // Nothing to normalize.
        BinaryOp::new(lhs_expr, operator, Operation::Binary(rhs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! binop {
        ($l:expr, $op:expr, $r:expr $(,)?) => {
            BinaryOp::new($l, $op, $r)
        };
    }

    macro_rules! assert_normalizes_to {
        ($op:expr, $expected:expr $(,)?) => {
            assert_eq!($op.normalize(), $expected);
        };
    }

    #[test]
    fn normalize_binary_op() {
        use BinaryOperator::{Div, Mod, Mul, Plus};

        assert_normalizes_to!(
            binop!(binop!(1, Plus, 2), Div, binop!(3, Mul, 4)),
            binop!(1, Plus, binop!(2, Div, binop!(3, Mul, 4))),
        );

        assert_normalizes_to!(
            binop!(binop!(1, Div, 2), Mul, binop!(3, Plus, binop!(4, Mod, 5))),
            binop!(binop!(binop!(1, Div, 2), Mul, 3), Plus, binop!(4, Mod, 5)),
        );

        assert_normalizes_to!(
            binop!(binop!(binop!(1, Plus, 2), Mul, 3), Div, 4),
            binop!(1, Plus, binop!(binop!(2, Mul, 3), Div, 4)),
        );

        assert_normalizes_to!(
            binop!(1, Div, binop!(binop!(2, Plus, 3), Mul, 4)),
            binop!(binop!(1, Div, 2), Plus, binop!(3, Mul, 4)),
        );
    }

    #[test]
    fn normalize_parenthesized() {
        use BinaryOperator::{Div, Mod, Mul, Plus};

        fn parens(op: BinaryOp) -> Expression {
            Expression::Parenthesis(Box::new(op.into()))
        }

        let op = binop!(
            parens(binop!(1, Div, 2)),
            Mul,
            parens(binop!(3, Mod, parens(binop!(4, Plus, 5)))),
        );

        assert_normalizes_to!(op.clone(), op);
    }

    #[test]
    fn already_normalized() {
        use BinaryOperator::Plus;

        let op = binop!(binop!(1, Plus, 2), Plus, binop!(3, Plus, 4));

        assert_normalizes_to!(op.clone(), op);
    }
}
