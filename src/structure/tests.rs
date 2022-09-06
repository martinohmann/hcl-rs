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
                    Identifier::new("EOS"),
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
