use super::*;
use crate::{BinaryOp, BinaryOperator, Conditional, Operation};
use std::fmt;

#[track_caller]
fn eval_to<T, U>(value: T, expected: U)
where
    T: Evaluate<Output = U> + fmt::Debug + PartialEq,
    U: fmt::Debug + PartialEq,
{
    let mut ctx = Context::new();
    assert_eq!(value.evaluate(&mut ctx).unwrap(), expected);
}

#[track_caller]
fn eval_error<T, E>(value: T, expected: E)
where
    T: Evaluate + fmt::Debug + PartialEq,
    <T as Evaluate>::Output: fmt::Debug,
    E: ToString,
{
    let mut ctx = Context::new();
    assert_eq!(
        value.evaluate(&mut ctx).unwrap_err().to_string(),
        expected.to_string()
    );
}

#[test]
fn eval_binary_op() {
    use {BinaryOperator::*, Operation::*};

    eval_to(
        BinaryOp::new(
            Binary(BinaryOp::new(1, Div, 2)),
            Mul,
            Binary(BinaryOp::new(3, Plus, Binary(BinaryOp::new(4, Div, 5)))),
        ),
        Expression::from(2.3),
    );
    eval_to(BinaryOp::new("foo", Eq, "foo"), Expression::from(true));
    eval_to(BinaryOp::new(false, Or, true), Expression::from(true));
    eval_to(BinaryOp::new(true, And, true), Expression::from(true));
    eval_to(BinaryOp::new(true, And, false), Expression::from(false));
    eval_to(BinaryOp::new(1, Less, 2), Expression::from(true));
    eval_to(
        BinaryOp::new(
            Binary(BinaryOp::new(1, Greater, 0)),
            And,
            Binary(BinaryOp::new("foo", NotEq, Expression::Null)),
        ),
        Expression::from(true),
    );
}

#[test]
fn eval_conditional() {
    eval_to(Conditional::new(true, "yes", "no"), Expression::from("yes"));
    eval_to(Conditional::new(false, "yes", "no"), Expression::from("no"));
    eval_error(
        Conditional::new("foo", "yes", "no"),
        "eval error: unexpected expression `\"foo\"`, expected a boolean",
    );
}
