use super::*;
use crate::{
    Attribute, BinaryOp, BinaryOperator, Block, BlockLabel, Body, Conditional, Expression, ForExpr,
    FuncCall, Heredoc, HeredocStripMode, Identifier, Object, ObjectKey, Operation, RawExpression,
    TemplateExpr, Traversal, TraversalOperator,
};
use pretty_assertions::assert_eq;
use serde_json::json;

#[track_caller]
fn expect_str<T: Serialize>(value: T, expected: &str) {
    assert_eq!(to_string(&value).unwrap(), expected);
}

#[test]
fn serialize_struct() {
    #[derive(serde::Serialize)]
    struct Test {
        foo: u32,
        bar: bool,
    }

    expect_str(Test { foo: 1, bar: true }, "foo = 1\nbar = true\n");
}

#[test]
fn serialize_tuple_struct() {
    #[derive(serde::Serialize)]
    struct Test1 {
        foo: u32,
    }

    #[derive(serde::Serialize)]
    struct Test2 {
        bar: &'static str,
    }

    #[derive(serde::Serialize)]
    struct TupleStruct(Test1, Test2);

    expect_str(
        TupleStruct(Test1 { foo: 1 }, Test2 { bar: "baz" }),
        "foo = 1\nbar = \"baz\"\n",
    );
}

#[test]
fn serialize_enum() {
    #[derive(serde::Serialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    #[derive(serde::Serialize, PartialEq, Debug)]
    struct Test {
        value: E,
    }

    expect_str(Test { value: E::Unit }, "value = \"Unit\"\n");
    expect_str(E::Newtype(1), "Newtype = 1\n");
    expect_str(E::Tuple(1, 2), "Tuple = [\n  1,\n  2\n]\n");
    expect_str(
        Test {
            value: E::Struct { a: 1 },
        },
        "value = {\n  \"Struct\" = {\n    \"a\" = 1\n  }\n}\n",
    );
}

#[test]
fn serialize_body() {
    let value = Body::builder()
        .add_attribute(("foo", 1u64))
        .add_attribute(("bar", "baz"))
        .add_block(
            Block::builder("qux")
                .add_attribute(("foo", "bar"))
                .add_block(
                    Block::builder("with_labels")
                        .add_label(BlockLabel::identifier("label1"))
                        .add_label("lab\"el2")
                        .add_attribute(("baz", vec![1u64, 2u64, 3u64]))
                        .build(),
                )
                .add_attribute(("an_object", {
                    Object::from([
                        (ObjectKey::identifier("foo"), Expression::from("bar")),
                        (
                            ObjectKey::from("enabled"),
                            Expression::from(RawExpression::new("var.enabled")),
                        ),
                        (
                            ObjectKey::Expression(RawExpression::from("var.name").into()),
                            Expression::from("the value"),
                        ),
                    ])
                }))
                .build(),
        )
        .build();

    let expected = r#"foo = 1
bar = "baz"

qux {
  foo = "bar"

  with_labels label1 "lab\"el2" {
    baz = [
      1,
      2,
      3
    ]
  }

  an_object = {
    foo = "bar"
    "enabled" = var.enabled
    var.name = "the value"
  }
}
"#;

    expect_str(value, expected);
}

#[test]
fn serialize_object() {
    let value = json!({
        "foo": [1, 2, 3],
        "bar": "baz",
        "qux": {
            "foo": "bar",
            "baz": "qux"
        }
    });

    let expected = r#"foo = [
  1,
  2,
  3
]
bar = "baz"
qux = {
  "foo" = "bar"
  "baz" = "qux"
}
"#;

    expect_str(value, expected);
}

#[test]
fn serialize_array() {
    let value = json!([
        {
            "foo": [1, 2, 3],
        },
        {
            "bar": "baz",
        },
        {
            "qux": {
                "foo": "bar",
                "baz": "qux"
            }
        }
    ]);

    let expected = r#"foo = [
  1,
  2,
  3
]
bar = "baz"
qux = {
  "foo" = "bar"
  "baz" = "qux"
}
"#;

    expect_str(value, expected);
}

#[test]
fn serialize_empty_block() {
    expect_str(Block::builder("empty").build(), "empty {}\n");
}

#[test]
fn serialize_traversal() {
    expect_str(
        Attribute::new(
            "attr",
            Traversal::new(
                Identifier::new("var"),
                [
                    TraversalOperator::GetAttr("foo".into()),
                    TraversalOperator::FullSplat,
                    TraversalOperator::GetAttr("bar".into()),
                    TraversalOperator::Index(1u64.into()),
                    TraversalOperator::AttrSplat,
                    TraversalOperator::GetAttr("baz".into()),
                    TraversalOperator::LegacyIndex(42),
                ],
            ),
        ),
        "attr = var.foo[*].bar[1].*.baz.42\n",
    );
}

#[test]
fn serialize_conditional() {
    expect_str(
        Attribute::new(
            "cond",
            Conditional::new(Identifier::new("cond_var"), "yes", "no"),
        ),
        "cond = cond_var ? \"yes\" : \"no\"\n",
    );
}

#[test]
fn serialize_operation() {
    expect_str(
        Attribute::new(
            "op",
            Operation::Binary(BinaryOp::new(1, BinaryOperator::Plus, 2)),
        ),
        "op = 1 + 2\n",
    );
}

#[test]
fn serialize_for_expr() {
    let body = Body::builder()
        .add_attribute((
            "list",
            ForExpr::new(
                Identifier::new("item"),
                Expression::Variable(Identifier::new("items")),
                FuncCall::builder("func")
                    .arg(Identifier::new("item"))
                    .build(),
            )
            .with_cond_expr(Identifier::new("item")),
        ))
        .add_attribute((
            "object",
            ForExpr::new(
                Identifier::new("value"),
                Expression::Variable(Identifier::new("items")),
                FuncCall::builder("tolower")
                    .arg(Identifier::new("value"))
                    .build(),
            )
            .with_key_var(Identifier::new("key"))
            .with_key_expr(
                FuncCall::builder("toupper")
                    .arg(Identifier::new("key"))
                    .build(),
            )
            .with_cond_expr(Operation::Binary(BinaryOp::new(
                Identifier::new("value"),
                BinaryOperator::NotEq,
                Expression::Null,
            )))
            .with_grouping(true),
        ))
        .build();

    let expected = r#"
list = [for item in items : func(item) if item]
object = {for key, value in items : toupper(key) => tolower(value)... if value != null}
"#
    .trim_start();

    expect_str(body, expected);
}

#[test]
fn serialize_func_call() {
    expect_str(
        Attribute::new("attr", FuncCall::new("foo")),
        "attr = foo()\n",
    );
    expect_str(
        Attribute::new("attr", FuncCall::builder("foo").arg(1).arg("two").build()),
        "attr = foo(1, \"two\")\n",
    );
    expect_str(
        Attribute::new(
            "attr",
            FuncCall::builder("foo")
                .arg(1)
                .arg(vec!["two", "three"])
                .expand_final(true)
                .build(),
        ),
        "attr = foo(1, [\"two\", \"three\"]...)\n",
    );
}

#[test]
fn serialize_heredoc() {
    let body = Body::builder()
        .add_block(
            Block::builder("content")
                .add_attribute((
                    "heredoc",
                    TemplateExpr::Heredoc(Heredoc::new(
                        Identifier::new("HEREDOC"),
                        "foo\n  bar\nbaz\n",
                    )),
                ))
                .add_attribute((
                    "heredoc_indent",
                    TemplateExpr::Heredoc(
                        Heredoc::new(Identifier::new("HEREDOC"), "    foo\n      bar\n    baz\n")
                            .with_strip_mode(HeredocStripMode::Indent),
                    ),
                ))
                .build(),
        )
        .build();

    let expected = r#"content {
  heredoc = <<HEREDOC
foo
  bar
baz
HEREDOC
  heredoc_indent = <<-HEREDOC
    foo
      bar
    baz
  HEREDOC
}
"#;

    expect_str(body, expected);
}

#[test]
fn serialize_errors() {
    assert!(to_string(&true).is_err());
    assert!(to_string("foo").is_err());
    assert!(to_string(&json!({"\"": "invalid attribute name"})).is_err())
}

#[test]
fn serialize_with_custom_formatter() {
    let body = Body::builder()
        .add_attribute(("foo", 1u64))
        .add_attribute(("bar", "baz"))
        .add_block(
            Block::builder("qux")
                .add_attribute(("foo", "bar"))
                .add_block(Block::builder("baz").add_attribute(("qux", true)).build())
                .add_attribute(("baz", "qux"))
                .build(),
        )
        .build();

    let default_expected = r#"foo = 1
bar = "baz"

qux {
  foo = "bar"

  baz {
    qux = true
  }

  baz = "qux"
}
"#;

    let custom_expected = r#"foo = 1
bar = "baz"
qux {
    foo = "bar"
    baz {
        qux = true
    }
    baz = "qux"
}
"#;

    expect_str(&body, default_expected);

    let mut buf = Vec::new();
    let formatter = Formatter::builder()
        .indent(b"    ")
        .dense(true)
        .build(&mut buf);
    let mut ser = Serializer::with_formatter(formatter);
    body.serialize(&mut ser).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), custom_expected);
}

#[test]
fn serialize_nested_expression() {
    expect_str(
        Attribute::new(
            "attr",
            Expression::Parenthesis(Box::new(Expression::Variable("foo".into()))),
        ),
        "attr = (foo)\n",
    );
}

#[test]
fn serialize_identifiers_with_hyphens() {
    expect_str(
        Attribute::new("hyphen-ated", Expression::Null),
        "hyphen-ated = null\n",
    );
}

#[test]
fn roundtrip() {
    let input = Body::builder()
        .add_block(
            Block::builder("resource")
                .add_label("aws_s3_bucket")
                .add_label("mybucket")
                .add_attribute((
                    "count",
                    Conditional::new(Traversal::new(Identifier::new("var"), ["enabled"]), 1, 0),
                ))
                .add_attribute(("bucket", "mybucket"))
                .add_attribute(("force_destroy", true))
                .add_block(
                    Block::builder("server_side_encryption_configuration")
                        .add_block(
                            Block::builder("rule")
                                .add_block(
                                    Block::builder("apply_server_side_encryption_by_default")
                                        .add_attribute((
                                            "kms_master_key_id",
                                            Traversal::new(
                                                Identifier::new("aws_kms_key"),
                                                ["mykey", "arn"],
                                            ),
                                        ))
                                        .add_attribute(("sse_algorithm", "aws:kms"))
                                        .build(),
                                )
                                .build(),
                        )
                        .build(),
                )
                .add_attribute((
                    "tags",
                    Expression::from_iter([
                        (
                            ObjectKey::from("application"),
                            Expression::String("myapp".into()),
                        ),
                        (
                            ObjectKey::Identifier("team".into()),
                            Expression::TemplateExpr(Box::new(TemplateExpr::QuotedString(
                                "${var.team}".into(),
                            ))),
                        ),
                        (
                            ObjectKey::Identifier("environment".into()),
                            Expression::Parenthesis(Box::new(Expression::Variable(
                                "environment".into(),
                            ))),
                        ),
                    ]),
                ))
                .build(),
        )
        .build();

    let serialized = to_string(&input).unwrap();

    let output: Body = crate::from_str(&serialized).unwrap();

    assert_eq!(input, output);
}
