use super::*;
use crate::{Block, Body, ElementAccess, Expression, Identifier, ObjectKey};
use pretty_assertions::assert_eq;
use serde::Deserialize;
use serde_json::{json, Value};

#[test]
fn deserialize_string_attribute() {
    let input = r#"foo = "bar""#;
    let expected: Value = json!({
        "foo": "bar"
    });
    assert_eq!(expected, from_str::<Value>(input).unwrap());
}

#[test]
fn deserialize_object() {
    let input = r#"foo = { bar = 42, "baz" = true }"#;
    let expected: Value = json!({
        "foo": {"bar": 42, "baz": true}
    });
    assert_eq!(expected, from_str::<Value>(input).unwrap());
}

#[test]
fn deserialze_block() {
    let input = r#"resource "aws_s3_bucket" "mybucket" { name = "mybucket" }"#;
    let expected: Value = json!({
        "resource": {
            "aws_s3_bucket": {
                "mybucket": {
                    "name": "mybucket"
                }
            }
        }
    });
    assert_eq!(expected, from_str::<Value>(input).unwrap());

    let input = r#"block { name = "asdf" }"#;
    let expected: Value = json!({
        "block": {
            "name": "asdf"
        }
    });
    assert_eq!(expected, from_str::<Value>(input).unwrap());
}

#[test]
fn deserialize_duplicate_block() {
    let input = r#"
        block {
          foo {
            bar = "baz"
          }

          foo {
            bar = 1
          }
        }

        other "one" "two" {
          foo = "bar"
        }

        other "two" "three" {
          bar = "baz"
        }
    "#;
    let expected = json!({
        "block": {
            "foo": [
                {
                    "bar": "baz"
                },
                {
                    "bar": 1
                }
            ]
        },
        "other": {
            "one": {
                "two": {
                    "foo": "bar"
                }
            },
            "two": {
                "three": {
                    "bar": "baz"
                }
            }
        }
    });
    assert_eq!(expected, from_str::<Value>(input).unwrap());

    let input = r#"
        foo { bar = "baz" }
        foo { bar = 1 }
    "#;
    let expected = json!({
        "foo": [
            {
                "bar": "baz"
            },
            {
                "bar": 1
            }
        ]
    });
    assert_eq!(expected, from_str::<Value>(input).unwrap());
}

#[test]
fn deserialize_duplicate_attribute() {
    let input = r#"
        foo = ["bar"]
        foo = ["baz"]
    "#;
    let expected = json!({"foo": ["baz"]});
    assert_eq!(expected, from_str::<Value>(input).unwrap());
}

#[test]
fn deserialize_duplicate_attribute_and_block() {
    let input = r#"
        foo = ["bar"]
        foo { bar = "baz" }
    "#;
    let expected = json!({"foo": {"bar": "baz"}});
    assert_eq!(expected, from_str::<Value>(input).unwrap());

    let input = r#"
        foo { bar = "baz" }
        foo = ["bar"]
    "#;
    let expected = json!({"foo": ["bar"]});
    assert_eq!(expected, from_str::<Value>(input).unwrap());
}

#[test]
fn deserialize_tuple() {
    let input = r#"foo = [true, 2, "three", var.enabled]"#;
    let expected: Value = json!({
        "foo": [true, 2, "three", "${var.enabled}"]
    });
    assert_eq!(expected, from_str::<Value>(input).unwrap());
}

#[test]
fn deserialize_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        foo: u32,
    }

    let input = r#"foo = 1"#;
    let expected = Test { foo: 1 };
    assert_eq!(expected, from_str::<Test>(input).unwrap());
}

#[test]
fn deserialize_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        value: E,
    }

    let input = r#"value = "Unit""#;
    let expected = Test { value: E::Unit };
    assert_eq!(expected, from_str::<Test>(input).unwrap());

    let input = r#"Newtype = 1"#;
    let expected = E::Newtype(1);
    assert_eq!(expected, from_str::<E>(input).unwrap());

    let input = r#"Tuple = [1,2]"#;
    let expected = E::Tuple(1, 2);
    assert_eq!(expected, from_str::<E>(input).unwrap());

    let input = r#"value = {"Struct" = {"a" = 1}}"#;
    let expected = Test {
        value: E::Struct { a: 1 },
    };
    assert_eq!(expected, from_str::<Test>(input).unwrap());
}

#[test]
fn deserialize_invalid_hcl() {
    let h = r#"invalid["#;
    assert!(from_str::<Value>(h).is_err());
}

#[test]
fn deserialize_terraform() {
    let input = r#"
        resource "aws_s3_bucket" "mybucket" {
          bucket        = "mybucket"
          force_destroy = true

          server_side_encryption_configuration {
            rule {
              apply_server_side_encryption_by_default {
                kms_master_key_id = aws_kms_key.mykey.arn
                sse_algorithm     = "aws:kms"
              }
            }
          }

          tags = {
            var.dynamic   = null
            "application" = "myapp"
            team          = "bar"
          }
        }"#;

    let expected = Body::builder()
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
                                            ElementAccess::new(
                                                Identifier::new("aws_kms_key"),
                                                "mykey",
                                            )
                                            .chain("arn"),
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
                            ObjectKey::RawExpression("var.dynamic".into()),
                            Expression::Null,
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

    let body: Body = from_str(input).unwrap();

    assert_eq!(expected, body);
}

// https://github.com/martinohmann/hcl-rs/issues/44
#[test]
fn issue_44() {
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[derive(Deserialize, Debug, PartialEq)]
    pub struct Config {
        #[serde(rename = "project")]
        pub projects: HashMap<String, Project>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    pub struct Project {
        pub proj_type: String,
        pub spec: Option<PathBuf>,
        pub dockerfile: Option<PathBuf>,
        pub scripts: Option<Vec<Script>>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    pub struct Script {
        pub name: String,
        pub command: String,
    }

    let input = r#"
        project "a" {
            proj_type = "generic"
            spec = "./test.spec"
        }
    "#;

    let config: Config = crate::from_str(input).unwrap();

    let expected = Config {
        projects: {
            let mut map = HashMap::new();
            map.insert(
                "a".into(),
                Project {
                    proj_type: "generic".into(),
                    spec: Some(PathBuf::from("./test.spec")),
                    dockerfile: None,
                    scripts: None,
                },
            );
            map
        },
    };

    assert_eq!(config, expected);
}
