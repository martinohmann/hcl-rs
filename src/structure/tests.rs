use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn body_into_value() {
    let body = Body::builder()
        .add_attribute(("foo", "bar"))
        .add_attribute(("bar", "baz"))
        .add_block(
            Block::builder("bar")
                .add_label("baz")
                .add_attribute(("foo", "bar"))
                .build(),
        )
        .add_block(
            Block::builder("bar")
                .add_label("qux")
                .add_attribute(("foo", 1))
                .build(),
        )
        .add_block(
            Block::builder("bar")
                .add_label("baz")
                .add_attribute(("bar", "baz"))
                .add_attribute(("baz", RawExpression::new("var.foo")))
                .build(),
        )
        .add_attribute(("foo", "baz"))
        .add_attribute((
            "heredoc",
            TemplateExpr::Heredoc(
                Heredoc::new(
                    Identifier::unchecked("EOS"),
                    "  foo \\\n  bar ${baz}\\\\backslash",
                )
                .with_strip_mode(HeredocStripMode::Indent),
            ),
        ))
        .build();

    let value = json!({
        "foo": "baz",
        "bar": {
            "baz": [
                {
                    "foo": "bar"
                },
                {
                    "bar": "baz",
                    "baz": "${var.foo}"
                }
            ],
            "qux": {
                "foo": 1
            },
        },
        "heredoc": "foo bar ${baz}\\backslash"
    });

    let expected: Value = serde_json::from_value(value).unwrap();

    assert_eq!(Value::from(body), expected);
}

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
