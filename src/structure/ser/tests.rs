use super::{
    attribute::AttributeSerializer,
    block::{BlockLabelSerializer, BlockSerializer},
    body::BodySerializer,
    conditional::ConditionalSerializer,
    expression::ExpressionSerializer,
    for_expr::ForExprSerializer,
    operation::OperationSerializer,
    template::TemplateExprSerializer,
};
use crate::structure::*;
use serde::{ser, Serialize};
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
    test_identity(BodySerializer, Body::default());
    test_identity(
        BodySerializer,
        Body::builder()
            .add_attribute(("foo", "bar"))
            .add_block(Block::builder("baz").build())
            .build(),
    );
    test_identity(AttributeSerializer, Attribute::new("foo", "bar"));
    test_identity(
        AttributeSerializer,
        Attribute::new("foo", vec!["bar", "baz"]),
    );
    test_identity(
        BlockSerializer,
        Block::builder("foo")
            .add_label("bar")
            .add_attribute(("baz", "qux"))
            .build(),
    );
    test_identity(BlockLabelSerializer, BlockLabel::string("foo"));
    test_identity(BlockLabelSerializer, BlockLabel::identifier("foo"));
    test_identity(ExpressionSerializer, Expression::Null);
    test_identity(ExpressionSerializer, Expression::Number(1.into()));
    test_identity(ExpressionSerializer, Expression::String("bar".into()));
    test_identity(
        ExpressionSerializer,
        Expression::from_iter([("foo", "bar")]),
    );
    test_identity(
        TemplateExprSerializer,
        TemplateExpr::QuotedString("${foo}".into()),
    );
    test_identity(
        TemplateExprSerializer,
        TemplateExpr::Heredoc(
            Heredoc::new(Identifier::new("EOS"), "  ${foo}")
                .with_strip_mode(HeredocStripMode::Indent),
        ),
    );
    test_identity(
        TemplateExprSerializer,
        TemplateExpr::Heredoc(Heredoc::new(Identifier::new("EOS"), "${foo}")),
    );
    test_identity(
        ConditionalSerializer,
        Conditional::new(Identifier::new("some_cond_var"), "yes", "no"),
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
        ForExpr::List(
            ForListExpr::new(
                Identifier::new("value"),
                vec![Expression::String(String::from("foo"))],
                Identifier::new("other_value"),
            )
            .with_index_var(Identifier::new("index"))
            .with_cond_expr(Expression::Bool(true)),
        ),
    );
    test_identity(
        ForExprSerializer,
        ForExpr::Object(
            ForObjectExpr::new(
                Identifier::new("value"),
                Expression::Object(Object::from([(
                    ObjectKey::from("k"),
                    Expression::String(String::from("v")),
                )])),
                Identifier::new("other_key"),
                Identifier::new("other_value"),
            )
            .with_key_var(Identifier::new("index"))
            .with_cond_expr(Expression::Bool(true))
            .with_grouping(true),
        ),
    );
}

#[test]
fn custom() {
    #[derive(Serialize)]
    struct CustomAttr {
        key: &'static str,
        #[serde(rename = "expr")]
        value: &'static str,
    }
    test_serialize(
        AttributeSerializer,
        CustomAttr {
            key: "foo",
            value: "bar",
        },
        Attribute::new("foo", "bar"),
    );
    test_serialize(
        AttributeSerializer,
        ("foo", "bar"),
        Attribute::new("foo", "bar"),
    );

    test_serialize(
        BlockSerializer,
        {
            let mut map = Map::new();
            map.insert("foo", (("bar", "baz"), ("qux", "foo")));
            map
        },
        Block::builder("foo")
            .add_attribute(("bar", "baz"))
            .add_attribute(("qux", "foo"))
            .build(),
    );

    #[derive(Serialize)]
    struct CustomBlock {
        #[serde(rename = "identifier")]
        ident: &'static str,
        body: Map<&'static str, &'static str>,
    }

    test_serialize(
        BlockSerializer,
        CustomBlock {
            ident: "foo",
            body: {
                let mut map = Map::new();
                map.insert("bar", "baz");
                map.insert("qux", "foo");
                map
            },
        },
        Block::builder("foo")
            .add_attribute(("bar", "baz"))
            .add_attribute(("qux", "foo"))
            .build(),
    );

    #[derive(Serialize)]
    struct CustomLabeledBlock {
        #[serde(rename = "identifier")]
        ident: &'static str,
        labels: [&'static str; 2],
        body: Map<&'static str, &'static str>,
    }

    test_serialize(
        BlockSerializer,
        CustomLabeledBlock {
            ident: "foo",
            labels: ["bar", "baz"],
            body: {
                let mut map = Map::new();
                map.insert("qux", "foo");
                map
            },
        },
        Block::builder("foo")
            .add_labels(["bar", "baz"])
            .add_attribute(("qux", "foo"))
            .build(),
    );

    test_serialize(
        ExpressionSerializer,
        Some(1u8),
        Expression::Number(1u8.into()),
    );

    test_serialize(
        ExpressionSerializer,
        Conditional::new(Identifier::new("some_cond_var"), "yes", "no"),
        Expression::from(Conditional::new(
            Identifier::new("some_cond_var"),
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
        TemplateExpr::Heredoc(Heredoc::new(Identifier::new("EOS"), "${foo}")),
        Expression::from(TemplateExpr::Heredoc(Heredoc::new(
            Identifier::new("EOS"),
            "${foo}",
        ))),
    );

    test_serialize(
        ExpressionSerializer,
        ForExpr::List(
            ForListExpr::new(
                Identifier::new("value"),
                vec![Expression::String(String::from("foo"))],
                Identifier::new("other_value"),
            )
            .with_index_var(Identifier::new("index"))
            .with_cond_expr(Expression::Bool(true)),
        ),
        Expression::from(ForExpr::List(
            ForListExpr::new(
                Identifier::new("value"),
                vec![Expression::String(String::from("foo"))],
                Identifier::new("other_value"),
            )
            .with_index_var(Identifier::new("index"))
            .with_cond_expr(Expression::Bool(true)),
        )),
    );

    test_serialize(
        ExpressionSerializer,
        ForListExpr::new(
            Identifier::new("value"),
            vec![Expression::String(String::from("foo"))],
            Identifier::new("other_value"),
        )
        .with_index_var(Identifier::new("index"))
        .with_cond_expr(Expression::Bool(true)),
        Expression::from(ForExpr::List(
            ForListExpr::new(
                Identifier::new("value"),
                vec![Expression::String(String::from("foo"))],
                Identifier::new("other_value"),
            )
            .with_index_var(Identifier::new("index"))
            .with_cond_expr(Expression::Bool(true)),
        )),
    );

    test_serialize(
        ConditionalSerializer,
        (
            Expression::VariableExpr(Identifier::new("some_cond_var")),
            Expression::String("yes".into()),
            Expression::String("no".into()),
        ),
        Conditional::new(Identifier::new("some_cond_var"), "yes", "no"),
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
