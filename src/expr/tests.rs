use super::{BinaryOp, BinaryOperator, Expression, Operation};
use pretty_assertions::assert_eq;

#[test]
fn normalize_binary_op() {
    use {BinaryOperator::*, Operation::*};

    let op = BinaryOp::new(
        Binary(BinaryOp::new(1, Plus, 2)),
        Div,
        Binary(BinaryOp::new(3, Mul, 4)),
    );

    let expected = BinaryOp::new(
        1,
        Plus,
        Binary(BinaryOp::new(2, Div, Binary(BinaryOp::new(3, Mul, 4)))),
    );

    assert_eq!(op.normalize(), expected);

    let op = BinaryOp::new(
        Binary(BinaryOp::new(1, Plus, 2)),
        Plus,
        Binary(BinaryOp::new(3, Plus, 4)),
    );

    let expected = op.clone();

    assert_eq!(op.normalize(), expected);

    let op = BinaryOp::new(
        Binary(BinaryOp::new(1, Div, 2)),
        Mul,
        Binary(BinaryOp::new(3, Plus, Binary(BinaryOp::new(4, Mod, 5)))),
    );

    let expected = BinaryOp::new(
        Binary(BinaryOp::new(Binary(BinaryOp::new(1, Div, 2)), Mul, 3)),
        Plus,
        Binary(BinaryOp::new(4, Mod, 5)),
    );

    assert_eq!(op.normalize(), expected);

    fn sub_expr(op: Operation) -> Expression {
        Expression::Parenthesis(Box::new(Expression::from(op)))
    }

    let op = BinaryOp::new(
        sub_expr(Binary(BinaryOp::new(1, Div, 2))),
        Mul,
        sub_expr(Binary(BinaryOp::new(
            3,
            Mod,
            sub_expr(Binary(BinaryOp::new(4, Plus, 5))),
        ))),
    );

    let expected = op.clone();

    assert_eq!(op.normalize(), expected);

    let op = BinaryOp::new(
        Binary(BinaryOp::new(Binary(BinaryOp::new(1, Plus, 2)), Mul, 3)),
        Div,
        4,
    );

    let expected = BinaryOp::new(
        1,
        Plus,
        Binary(BinaryOp::new(Binary(BinaryOp::new(2, Mul, 3)), Div, 4)),
    );

    assert_eq!(op.normalize(), expected);

    let op = BinaryOp::new(
        1,
        Div,
        Binary(BinaryOp::new(Binary(BinaryOp::new(2, Plus, 3)), Mul, 4)),
    );

    let expected = BinaryOp::new(
        Binary(BinaryOp::new(1, Div, 2)),
        Plus,
        Binary(BinaryOp::new(3, Mul, 4)),
    );

    assert_eq!(op.normalize(), expected);
}
