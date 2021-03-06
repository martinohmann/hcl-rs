use super::*;
use crate::{Block, BlockLabel, Body, Expression, Object, ObjectKey, RawExpression};
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn serialize_struct() {
    #[derive(serde::Serialize)]
    struct Test {
        foo: u32,
        bar: bool,
    }

    let v = Test { foo: 1, bar: true };
    let expected = "foo = 1\nbar = true\n";
    assert_eq!(&to_string(&v).unwrap(), expected);
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

    let v = TupleStruct(Test1 { foo: 1 }, Test2 { bar: "baz" });
    let expected = "foo = 1\nbar = \"baz\"\n";
    assert_eq!(&to_string(&v).unwrap(), expected);
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

    let v = Test { value: E::Unit };
    let expected = "value = \"Unit\"\n";
    assert_eq!(&to_string(&v).unwrap(), expected);

    let v = E::Newtype(1);
    let expected = "Newtype = 1\n";
    assert_eq!(&to_string(&v).unwrap(), expected);

    let v = E::Tuple(1, 2);
    let expected = "Tuple = [\n  1,\n  2\n]\n";
    assert_eq!(&to_string(&v).unwrap(), expected);

    let v = Test {
        value: E::Struct { a: 1 },
    };
    let expected = "value = {\n  \"Struct\" = {\n    \"a\" = 1\n  }\n}\n";
    assert_eq!(&to_string(&v).unwrap(), expected);
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
                    let mut object = Object::new();

                    object.insert(ObjectKey::identifier("foo"), "bar".into());
                    object.insert(
                        ObjectKey::string("enabled"),
                        RawExpression::new("var.enabled").into(),
                    );
                    object.insert(ObjectKey::raw_expression("var.name"), "the value".into());
                    object
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
    "${var.name}" = "the value"
  }
}
"#;

    assert_eq!(to_string(&value).unwrap(), expected);
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

    assert_eq!(to_string(&value).unwrap(), expected);
}

#[test]
fn serialize_array() {
    let value = json!([
        {
            "foo": [1, 2, 3],
        },
        {
            "bar": "baz",
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

    assert_eq!(to_string(&value).unwrap(), expected);
}

#[test]
fn serialize_empty_block() {
    let body = Body::builder()
        .add_block(Block::builder("empty").build())
        .build();

    assert_eq!(to_string(&body).unwrap(), "empty {}\n");
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

    assert_eq!(to_string(&body).unwrap(), default_expected);

    let formatter = PrettyFormatter::builder()
        .indent(b"    ")
        .dense(true)
        .build();
    let mut buf = Vec::new();
    let mut ser = Serializer::with_formatter(&mut buf, formatter);
    body.serialize(&mut ser).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), custom_expected);
}

#[test]
fn roundtrip() {
    let input = Body::builder()
        .add_block(
            Block::builder("resource")
                .add_label("aws_s3_bucket")
                .add_label("mybucket")
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
                                            RawExpression::new("aws_kms_key.mykey.arn"),
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
                            ObjectKey::String("${var.dynamic}".into()),
                            Expression::Bool(true),
                        ),
                        (
                            ObjectKey::String("application".into()),
                            Expression::String("myapp".into()),
                        ),
                        (
                            ObjectKey::Identifier("team".into()),
                            Expression::String("bar".into()),
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
