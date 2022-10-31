use super::{
    ConditionalSerializer, ExpressionSerializer, ForExprSerializer, OperationSerializer,
    TemplateExprSerializer,
};
use crate::expr::*;
use crate::Identifier;
use serde::ser;
use std::fmt::Debug;

#[track_caller]
fn test_identity<S, T>(ser: S, value: T)
where
    S: ser::Serializer<Ok = T>,
    T: ser::Serialize + PartialEq + Debug,
{
    assert_eq!(value, value.serialize(ser).unwrap());
}

#[track_caller]
fn test_serialize<S, G, E>(ser: S, given: G, expected: E)
where
    S: ser::Serializer<Ok = E>,
    G: ser::Serialize,
    E: PartialEq + Debug,
{
    assert_eq!(expected, given.serialize(ser).unwrap());
}

#[test]
fn identity() {
    test_identity(ExpressionSerializer, Expression::Null);
    test_identity(ExpressionSerializer, Expression::Number(1.into()));
    test_identity(ExpressionSerializer, Expression::String("bar".into()));
    test_identity(
        ExpressionSerializer,
        Expression::from_iter([("foo", "bar")]),
    );
    test_identity(TemplateExprSerializer, TemplateExpr::from("${foo}"));
    test_identity(
        TemplateExprSerializer,
        TemplateExpr::Heredoc(
            Heredoc::new(Identifier::unchecked("EOS"), "  ${foo}")
                .with_strip_mode(HeredocStripMode::Indent),
        ),
    );
    test_identity(
        TemplateExprSerializer,
        TemplateExpr::Heredoc(Heredoc::new(Identifier::unchecked("EOS"), "${foo}")),
    );
    test_identity(
        ConditionalSerializer,
        Conditional::new(Variable::unchecked("some_cond_var"), "yes", "no"),
    );
    test_identity(
        OperationSerializer,
        Operation::Unary(UnaryOp::new(UnaryOperator::Neg, 1)),
    );
    test_identity(
        OperationSerializer,
        Operation::Binary(BinaryOp::new(1, BinaryOperator::Plus, 1)),
    );
    test_identity(
        ForExprSerializer,
        ForExpr::new(
            Identifier::unchecked("value"),
            vec![Expression::String(String::from("foo"))],
            Variable::unchecked("other_value"),
        )
        .with_key_var(Identifier::unchecked("index"))
        .with_cond_expr(Expression::Bool(true)),
    );
    test_identity(
        ForExprSerializer,
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::Object(Object::from([(
                ObjectKey::from("k"),
                Expression::String(String::from("v")),
            )])),
            Variable::unchecked("other_value"),
        )
        .with_key_var(Identifier::unchecked("index"))
        .with_key_expr(Variable::unchecked("other_key"))
        .with_cond_expr(Expression::Bool(true))
        .with_grouping(true),
    );
}

#[test]
fn custom() {
    test_serialize(
        ExpressionSerializer,
        Some(1u8),
        Expression::Number(1u8.into()),
    );

    test_serialize(
        ExpressionSerializer,
        Conditional::new(Variable::unchecked("some_cond_var"), "yes", "no"),
        Expression::from(Conditional::new(
            Variable::unchecked("some_cond_var"),
            "yes",
            "no",
        )),
    );

    test_serialize(
        ExpressionSerializer,
        Operation::Unary(UnaryOp::new(UnaryOperator::Neg, 1)),
        Expression::from(Operation::Unary(UnaryOp::new(UnaryOperator::Neg, 1))),
    );

    test_serialize(
        ExpressionSerializer,
        TemplateExpr::Heredoc(Heredoc::new(Identifier::unchecked("EOS"), "${foo}")),
        Expression::from(TemplateExpr::Heredoc(Heredoc::new(
            Identifier::unchecked("EOS"),
            "${foo}",
        ))),
    );

    test_serialize(
        ExpressionSerializer,
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

    test_serialize(
        ExpressionSerializer,
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

    test_serialize(
        ConditionalSerializer,
        (
            Expression::from(Variable::unchecked("some_cond_var")),
            Expression::String("yes".into()),
            Expression::String("no".into()),
        ),
        Conditional::new(Variable::unchecked("some_cond_var"), "yes", "no"),
    );

    test_serialize(
        OperationSerializer,
        ("-", Expression::Number(1.into())),
        Operation::Unary(UnaryOp::new(UnaryOperator::Neg, 1)),
    );

    test_serialize(
        OperationSerializer,
        (
            Expression::Number(1.into()),
            "+",
            Expression::Number(1.into()),
        ),
        Operation::Binary(BinaryOp::new(1, BinaryOperator::Plus, 1)),
    );
}
