use super::*;
use crate::{
    BinaryOp, BinaryOperator, Conditional, ForExpr, ForIntro, ForListExpr, ForObjectExpr,
    Identifier, Operation,
};
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

#[test]
fn eval_for_expr() {
    eval_to(
        ForExpr::List(
            ForListExpr::new(
                ForIntro::new(
                    Identifier::new("item"),
                    Expression::from_iter([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
                ),
                Operation::Binary(BinaryOp::new(
                    Expression::VariableExpr(Identifier::new("item")),
                    BinaryOperator::Mul,
                    2,
                )),
            )
            .with_cond(Operation::Binary(BinaryOp::new(
                Expression::VariableExpr(Identifier::new("item")),
                BinaryOperator::Less,
                5.0,
            ))),
        ),
        Expression::from_iter([2.0, 4.0, 6.0, 8.0]),
    );

    eval_to(
        ForExpr::Object(
            ForObjectExpr::new(
                ForIntro::new(
                    Identifier::new("value"),
                    Expression::from_iter([("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]),
                )
                .with_key(Identifier::new("key")),
                Expression::VariableExpr(Identifier::new("value")),
                Expression::VariableExpr(Identifier::new("key")),
            )
            .with_cond(Operation::Binary(BinaryOp::new(
                Expression::VariableExpr(Identifier::new("key")),
                BinaryOperator::NotEq,
                Expression::from("d"),
            ))),
        ),
        Expression::from_iter([("1", "a"), ("2", "b"), ("3", "c")]),
    );

    eval_to(
        ForExpr::Object(
            ForObjectExpr::new(
                ForIntro::new(
                    Identifier::new("value"),
                    Expression::from_iter([("a", 1), ("b", 2), ("c", 3), ("d", 4)]),
                )
                .with_key(Identifier::new("key")),
                Expression::from("foo"),
                Expression::VariableExpr(Identifier::new("value")),
            )
            .with_cond(Operation::Binary(BinaryOp::new(
                Expression::VariableExpr(Identifier::new("key")),
                BinaryOperator::NotEq,
                Expression::from("d"),
            )))
            .with_value_grouping(true),
        ),
        Expression::from_iter([("foo", vec![1, 2, 3])]),
    );
}
