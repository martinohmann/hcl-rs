use super::*;
use crate::{
    BinaryOp, BinaryOperator, Conditional, ForExpr, Identifier, Operation, Traversal,
    TraversalOperator,
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
fn eval_error<T>(value: T, expected: EvalErrorKind)
where
    T: Evaluate + fmt::Debug + PartialEq,
    <T as Evaluate>::Output: fmt::Debug,
{
    let mut ctx = Context::new();
    assert_eq!(value.evaluate(&mut ctx).unwrap_err().kind(), &expected);
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
        EvalErrorKind::Unexpected(Expression::from("foo"), "a boolean"),
    );
}

#[test]
fn eval_for_expr() {
    eval_to(
        ForExpr::new(
            Identifier::new("item"),
            Expression::from_iter([1, 2, 3, 4, 5, 6, 7]),
            Operation::Binary(BinaryOp::new(
                Expression::Variable(Identifier::new("item")),
                BinaryOperator::Mul,
                2,
            )),
        )
        .with_cond_expr(Operation::Binary(BinaryOp::new(
            Expression::Variable(Identifier::new("item")),
            BinaryOperator::Less,
            5,
        ))),
        Expression::from_iter([2, 4, 6, 8]),
    );

    eval_to(
        ForExpr::new(
            Identifier::new("value"),
            Expression::from_iter([("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]),
            Expression::Variable(Identifier::new("key")),
        )
        .with_key_var(Identifier::new("key"))
        .with_key_expr(Expression::Variable(Identifier::new("value")))
        .with_cond_expr(Operation::Binary(BinaryOp::new(
            Expression::Variable(Identifier::new("key")),
            BinaryOperator::NotEq,
            Expression::from("d"),
        ))),
        Expression::from_iter([("1", "a"), ("2", "b"), ("3", "c")]),
    );

    eval_to(
        ForExpr::new(
            Identifier::new("value"),
            Expression::from_iter(["a", "b", "c", "d"]),
            Expression::Variable(Identifier::new("value")),
        )
        .with_key_var(Identifier::new("index"))
        .with_key_expr(Expression::Variable(Identifier::new("index"))),
        Expression::from_iter([(0, "a"), (1, "b"), (2, "c"), (3, "d")]),
    );

    eval_to(
        ForExpr::new(
            Identifier::new("value"),
            Expression::from_iter([("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]),
            Expression::Variable(Identifier::new("key")),
        )
        .with_key_var(Identifier::new("key")),
        Expression::from_iter(["a", "b", "c", "d"]),
    );

    eval_to(
        ForExpr::new(
            Identifier::new("value"),
            Expression::from_iter(["a", "b", "c", "d"]),
            Expression::Variable(Identifier::new("value")),
        )
        .with_key_var(Identifier::new("index"))
        .with_key_expr(Expression::Variable(Identifier::new("index"))),
        Expression::from_iter([(0, "a"), (1, "b"), (2, "c"), (3, "d")]),
    );

    eval_to(
        ForExpr::new(
            Identifier::new("value"),
            Expression::from_iter([("a", 1), ("b", 2), ("c", 3), ("d", 4)]),
            Expression::Variable(Identifier::new("value")),
        )
        .with_key_var(Identifier::new("key"))
        .with_key_expr(Expression::from("foo"))
        .with_cond_expr(Operation::Binary(BinaryOp::new(
            Expression::Variable(Identifier::new("key")),
            BinaryOperator::NotEq,
            Expression::from("d"),
        )))
        .with_grouping(true),
        Expression::from_iter([("foo", vec![1, 2, 3])]),
    );
}

#[test]
fn eval_traversal() {
    use TraversalOperator::*;

    // legacy index access: expr.2
    eval_to(
        Traversal::new(vec![1, 2, 3], [LegacyIndex(1)]),
        expression!(2),
    );

    // legacy index access: expr[2]
    eval_to(
        Traversal::new(vec![1, 2, 3], [Index(Expression::from(2))]),
        expression!(3),
    );

    // get-attr: expr.foo
    eval_to(
        Traversal::new(
            expression!({"foo" = [1, 2, 3], "bar" = []}),
            [GetAttr(Identifier::new("foo"))],
        ),
        expression!([1, 2, 3]),
    );

    // chain get-attr -> index: expr.foo[2]
    eval_to(
        Traversal::new(
            Traversal::new(
                expression!({"foo" = [1, 2, 3], "bar" = []}),
                [GetAttr(Identifier::new("foo"))],
            ),
            [Index(Expression::from(2))],
        ),
        expression!(3),
    );

    // full-splat non-array
    eval_to(
        Traversal::new(
            expression!({"foo" = [1, 2, 3], "bar" = []}),
            [FullSplat, GetAttr(Identifier::new("foo"))],
        ),
        expression!([[1, 2, 3]]),
    );

    // full-splat array
    eval_to(
        Traversal::new(
            expression! {
                [
                    { "foo" = 2 },
                    { "foo" = 1, "bar" = 2 }
                ]
            },
            [FullSplat, GetAttr(Identifier::new("foo"))],
        ),
        expression!([2, 1]),
    );

    // full-splat null
    eval_to(
        Traversal::new(
            Expression::Null,
            [FullSplat, GetAttr(Identifier::new("foo"))],
        ),
        expression!([]),
    );

    // attr-splat non-array
    eval_to(
        Traversal::new(
            expression!({"foo" = [1, 2, 3], "bar" = []}),
            [AttrSplat, GetAttr(Identifier::new("foo"))],
        ),
        expression!([[1, 2, 3]]),
    );

    // attr-splat array
    eval_to(
        Traversal::new(
            expression! {
                [
                    { "foo" = 2 },
                    { "foo" = 1, "bar" = 2 }
                ]
            },
            [AttrSplat, GetAttr(Identifier::new("foo"))],
        ),
        expression!([2, 1]),
    );

    // attr-splat null
    eval_to(
        Traversal::new(
            Expression::Null,
            [AttrSplat, GetAttr(Identifier::new("foo"))],
        ),
        expression!([]),
    );

    // attr-splat followed by non-get-attr
    eval_to(
        Traversal::new(
            expression! {
                [
                    { "foo" = { "bar" = [1, 2, 3] } },
                    { "foo" = { "bar" = [10, 20, 30] } }
                ]
            },
            [
                AttrSplat,
                GetAttr(Identifier::new("foo")),
                GetAttr(Identifier::new("bar")),
                Index(expression!(1)),
            ],
        ),
        expression!([[1, 2, 3], [10, 20, 30]]),
    );

    // full-splat followed by non-get-attr
    eval_to(
        Traversal::new(
            expression! {
                [
                    { "foo" = { "bar" = [1, 2, 3] } },
                    { "foo" = { "bar" = [10, 20, 30] } }
                ]
            },
            [
                FullSplat,
                GetAttr(Identifier::new("foo")),
                GetAttr(Identifier::new("bar")),
                Index(expression!(1)),
            ],
        ),
        expression!([2, 20]),
    );

    // errors
    eval_error(
        Traversal::new(vec![1, 2, 3], [LegacyIndex(5)]),
        EvalErrorKind::IndexOutOfBounds(5),
    );
}
