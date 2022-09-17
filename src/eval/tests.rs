use super::*;
use crate::{BinaryOp, BinaryOperator, Operation};
use std::fmt;

#[track_caller]
fn evaluates_to<T, U>(value: T, expected: U)
where
    T: Evaluate<Output = U> + fmt::Debug + PartialEq,
    U: fmt::Debug + PartialEq,
{
    let mut ctx = Context::new();
    assert_eq!(value.evaluate(&mut ctx).unwrap(), expected);
}

#[test]
fn eval_binary_op() {
    use {BinaryOperator::*, Operation::*};

    evaluates_to(
        BinaryOp::new(
            Binary(BinaryOp::new(1, Div, 2)),
            Mul,
            Binary(BinaryOp::new(3, Plus, Binary(BinaryOp::new(4, Div, 5)))),
        ),
        Expression::from(2.3),
    );
    evaluates_to(BinaryOp::new("foo", Eq, "foo"), Expression::from(true));
    evaluates_to(BinaryOp::new(false, Or, true), Expression::from(true));
    evaluates_to(BinaryOp::new(true, And, true), Expression::from(true));
    evaluates_to(BinaryOp::new(true, And, false), Expression::from(false));
    evaluates_to(BinaryOp::new(1, Less, 2), Expression::from(true));
    evaluates_to(
        BinaryOp::new(
            Binary(BinaryOp::new(1, Greater, 0)),
            And,
            Binary(BinaryOp::new("foo", NotEq, Expression::Null)),
        ),
        Expression::from(true),
    );
}
