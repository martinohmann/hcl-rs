use crate::expr::*;
use crate::Identifier;
use serde::ser;

#[track_caller]
fn assert_expr<G>(given: G, expected: Expression)
where
    G: ser::Serialize,
{
    assert_eq!(Expression::from_serializable(&given).unwrap(), expected);
}

#[test]
fn roundtrip() {
    assert_expr(Expression::Null, Expression::Null);
    assert_expr(Expression::Number(1.into()), Expression::Number(1.into()));
    assert_expr(
        Expression::String("bar".into()),
        Expression::String("bar".into()),
    );
    assert_expr(
        Expression::from_iter([("foo", "bar")]),
        Expression::from_iter([("foo", "bar")]),
    );
    assert_expr(
        Expression::from(Variable::unchecked("var")),
        Expression::from(Variable::unchecked("var")),
    )
}

#[test]
fn builtin() {
    assert_expr(Some(1u8), Expression::Number(1u8.into()));

    assert_expr(
        Conditional::new(Variable::unchecked("some_cond_var"), "yes", "no"),
        Expression::from(Conditional::new(
            Variable::unchecked("some_cond_var"),
            "yes",
            "no",
        )),
    );

    assert_expr(
        Operation::Unary(UnaryOp::new(UnaryOperator::Neg, 1)),
        Expression::from(Operation::Unary(UnaryOp::new(UnaryOperator::Neg, 1))),
    );

    assert_expr(
        TemplateExpr::Heredoc(Heredoc::new(Identifier::unchecked("EOS"), "${foo}")),
        Expression::from(TemplateExpr::Heredoc(Heredoc::new(
            Identifier::unchecked("EOS"),
            "${foo}",
        ))),
    );

    assert_expr(
        ForExpr::new(
            Identifier::unchecked("value"),
            vec![Expression::String(String::from("foo"))],
            Variable::unchecked("other_value"),
        )
        .with_key_var(Identifier::unchecked("index"))
        .with_cond_expr(Expression::Bool(true)),
        Expression::from(
            ForExpr::new(
                Identifier::unchecked("value"),
                vec![Expression::String(String::from("foo"))],
                Variable::unchecked("other_value"),
            )
            .with_key_var(Identifier::unchecked("index"))
            .with_cond_expr(Expression::Bool(true)),
        ),
    );

    assert_expr(
        ForExpr::new(
            Identifier::unchecked("value"),
            vec![Expression::String(String::from("foo"))],
            Variable::unchecked("other_value"),
        )
        .with_key_var(Identifier::unchecked("key"))
        .with_key_expr(Variable::unchecked("key"))
        .with_cond_expr(Expression::Bool(true)),
        Expression::from(
            ForExpr::new(
                Identifier::unchecked("value"),
                vec![Expression::String(String::from("foo"))],
                Variable::unchecked("other_value"),
            )
            .with_key_var(Identifier::unchecked("key"))
            .with_key_expr(Variable::unchecked("key"))
            .with_cond_expr(Expression::Bool(true)),
        ),
    );

    assert_expr(
        Traversal::builder(Variable::unchecked("some_var"))
            .index(0)
            .build(),
        Expression::from(
            Traversal::builder(Variable::unchecked("some_var"))
                .index(0)
                .build(),
        ),
    );

    assert_expr(
        FuncCall::builder("func").arg(0).build(),
        Expression::from(FuncCall::builder("func").arg(0).build()),
    );

    assert_expr(
        Operation::Unary(UnaryOp::new(UnaryOperator::Neg, 1)),
        Expression::from(Operation::Unary(UnaryOp::new(UnaryOperator::Neg, 1))),
    );

    assert_expr(
        Operation::Binary(BinaryOp::new(1, BinaryOperator::Plus, 1)),
        Expression::from(Operation::Binary(BinaryOp::new(1, BinaryOperator::Plus, 1))),
    );

    assert_expr(
        TemplateExpr::from("Hello ${world}!"),
        Expression::from(TemplateExpr::from("Hello ${world}!")),
    );

    assert_expr(
        Variable::unchecked("var"),
        Expression::from(Variable::unchecked("var")),
    );

    assert_expr(
        RawExpression::new("raw"),
        Expression::from(RawExpression::new("raw")),
    );
}

#[test]
fn custom() {
    assert_expr((), Expression::Null);
    assert_expr(1, Expression::Number(1.into()));
    assert_expr("bar", Expression::String("bar".into()));
    assert_expr(["foo", "bar"], Expression::from_iter(["foo", "bar"]));
}
