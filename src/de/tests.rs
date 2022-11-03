use super::*;
use crate::expr::{
    BinaryOp, BinaryOperator, Expression, ForExpr, FuncCall, ObjectKey, Operation, Traversal,
    TraversalOperator, UnaryOp, UnaryOperator, Variable,
};
use crate::structure::{Block, Body};
use crate::{value, Identifier, Value};
use pretty_assertions::assert_eq;
use serde::Deserialize;
use std::fmt::Debug;

#[track_caller]
fn expect_value<'de, T>(input: &'de str, expected: T)
where
    T: Deserialize<'de> + Debug + PartialEq,
{
    assert_eq!(from_str::<T>(input).unwrap(), expected);
}

#[test]
fn deserialize_string_attribute() {
    expect_value(r#"foo = "bar""#, value!({ foo = "bar" }))
}

#[test]
fn deserialize_object() {
    expect_value(
        r#"foo = { bar = 42, "baz" = true }"#,
        value!({ foo = { bar = 42, baz = true } }),
    )
}

#[test]
fn deserialze_block() {
    expect_value(
        r#"resource "aws_s3_bucket" "mybucket" { name = "mybucket" }"#,
        value!({ resource = { aws_s3_bucket = { mybucket = { name = "mybucket" } } } }),
    );

    expect_value(
        r#"block { name = "asdf" }"#,
        value!({ block = { name = "asdf" } }),
    );
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
    let expected = value!({
        block = {
            foo = [
                { "bar" = "baz" },
                { "bar" = 1 }
            ]
        },
        other = {
            one = {
                two = {
                    foo = "bar"
                }
            },
            two = {
                three = {
                    bar = "baz"
                }
            }
        }
    });
    expect_value(input, expected);

    let input = r#"
        foo { bar = "baz" }
        foo { bar = 1 }
    "#;
    let expected = value!({ foo = [{ bar = "baz" }, { bar = 1 }] });
    expect_value(input, expected);
}

#[test]
fn deserialize_duplicate_attribute() {
    let input = r#"
        foo = ["bar"]
        foo = ["baz"]
    "#;
    expect_value(input, value!({ foo = ["baz"] }));
}

#[test]
fn deserialize_duplicate_attribute_and_block() {
    let input = r#"
        foo = ["bar"]
        foo { bar = "baz" }
    "#;
    expect_value(input, value!({ foo = { bar = "baz" } }));

    let input = r#"
        foo { bar = "baz" }
        foo = ["bar"]
    "#;
    expect_value(input, value!({ foo = ["bar"] }));
}

#[test]
fn deserialize_tuple() {
    let input = r#"foo = [true, 2, "three", var.enabled]"#;
    let expected = value!({ foo = [true, 2, "three", "${var.enabled}"] });
    expect_value(input, expected);
}

#[test]
fn deserialize_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        foo: u32,
    }

    expect_value(r#"foo = 1"#, Test { foo: 1 });
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

    expect_value(r#"value = "Unit""#, Test { value: E::Unit });
    expect_value(r#"Newtype = 1"#, E::Newtype(1));
    expect_value(r#"Tuple = [1,2]"#, E::Tuple(1, 2));
    expect_value(
        r#"value = {"Struct" = {"a" = 1}}"#,
        Test {
            value: E::Struct { a: 1 },
        },
    );
}

#[test]
fn deserialize_func_call() {
    let input = r#"attr = foo(1, "bar", ["baz", "qux"]...)"#;
    let expected = Body::builder()
        .add_attribute((
            "attr",
            FuncCall::builder("foo")
                .arg(1)
                .arg("bar")
                .arg(vec!["baz", "qux"])
                .expand_final(true)
                .build(),
        ))
        .build();

    expect_value(input, expected);
}

#[test]
fn deserialize_operation() {
    let input = r#"
        unary = !variable
        binary = 1 + 1
    "#;

    let expected = Body::builder()
        .add_attribute((
            "unary",
            Operation::Unary(UnaryOp::new(
                UnaryOperator::Not,
                Variable::unchecked("variable"),
            )),
        ))
        .add_attribute((
            "binary",
            Operation::Binary(BinaryOp::new(1, BinaryOperator::Plus, 1)),
        ))
        .build();

    expect_value(input, expected);
}

#[test]
fn deserialize_for_expr() {
    let input = r#"
        list = [for item in items : func(item) if item]
        object = {for key, value in items : toupper(key) => tolower(value)...}
    "#;

    let expected = Body::builder()
        .add_attribute((
            "list",
            ForExpr::new(
                Identifier::unchecked("item"),
                Variable::unchecked("items"),
                FuncCall::builder("func")
                    .arg(Variable::unchecked("item"))
                    .build(),
            )
            .with_cond_expr(Variable::unchecked("item")),
        ))
        .add_attribute((
            "object",
            ForExpr::new(
                Identifier::unchecked("value"),
                Variable::unchecked("items"),
                FuncCall::builder("tolower")
                    .arg(Variable::unchecked("value"))
                    .build(),
            )
            .with_key_var(Identifier::unchecked("key"))
            .with_key_expr(
                FuncCall::builder("toupper")
                    .arg(Variable::unchecked("key"))
                    .build(),
            )
            .with_grouping(true),
        ))
        .build();

    expect_value(input, expected);
}

#[test]
fn deserialize_invalid_hcl() {
    assert!(from_str::<Value>(r#"invalid["#).is_err());
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
                                            Traversal::new(
                                                Variable::unchecked("aws_kms_key"),
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
                            ObjectKey::from(Traversal::new(
                                Variable::unchecked("var"),
                                ["dynamic"],
                            )),
                            Expression::Null,
                        ),
                        (
                            ObjectKey::from("application"),
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

    expect_value(input, expected);
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

    expect_value(input, expected);
}

#[test]
fn issue_66() {
    let expected = Body::builder()
        .add_attribute((
            "a",
            Traversal::new(Variable::unchecked("b"), [Expression::from("c")]),
        ))
        .build();

    expect_value("a = b[\"c\"]", expected);
}

#[test]
fn issue_81() {
    let input = r#"
        attr_splat = module.instance.*.id
        full_splat = module.instance[*].id
    "#;

    let expected = Body::builder()
        .add_attribute((
            "attr_splat",
            Traversal::new(
                Variable::unchecked("module"),
                [
                    TraversalOperator::GetAttr("instance".into()),
                    TraversalOperator::AttrSplat,
                    TraversalOperator::GetAttr("id".into()),
                ],
            ),
        ))
        .add_attribute((
            "full_splat",
            Traversal::new(
                Variable::unchecked("module"),
                [
                    TraversalOperator::GetAttr("instance".into()),
                    TraversalOperator::FullSplat,
                    TraversalOperator::GetAttr("id".into()),
                ],
            ),
        ))
        .build();

    expect_value(input, expected);
}

#[test]
fn issue_83() {
    let expected = Body::builder()
        .add_attribute((
            "attr",
            Traversal::new(
                Variable::unchecked("module"),
                [
                    TraversalOperator::GetAttr("instance".into()),
                    TraversalOperator::LegacyIndex(0),
                    TraversalOperator::GetAttr("id".into()),
                ],
            ),
        ))
        .build();

    expect_value("attr = module.instance.0.id", expected);
}
