mod common;

use common::assert_serialize;
use hcl::{
    expr::{BinaryOp, BinaryOperator, Conditional, Expression, Traversal, Variable},
    ser::{block, doubly_labeled_block, labeled_block},
    Map,
};
use indexmap::indexmap;
use indoc::indoc;
use serde::Serialize;

#[test]
fn custom_block() {
    #[derive(Serialize)]
    struct Config {
        #[serde(serialize_with = "block")]
        block: Map<&'static str, &'static str>,
    }

    let config = Config {
        block: indexmap! { "a" => "b", "c" => "d" },
    };

    let expected = indoc! {r#"
        block {
          a = "b"
          c = "d"
        }
    "#};

    assert_serialize(config, expected);
}

#[test]
fn custom_labeled_block() {
    #[derive(Serialize)]
    struct Config {
        #[serde(serialize_with = "labeled_block")]
        block: Map<&'static str, Map<&'static str, &'static str>>,
    }

    let config = Config {
        block: indexmap! {
            "one" => indexmap! { "a" => "b" },
            "two" => indexmap! { "c" => "d" },
        },
    };

    let expected = indoc! {r#"
        block "one" {
          a = "b"
        }

        block "two" {
          c = "d"
        }
    "#};

    assert_serialize(config, expected);
}

#[test]
fn custom_doubly_labeled_block() {
    #[derive(Serialize)]
    struct Config {
        #[serde(serialize_with = "doubly_labeled_block")]
        block: Map<&'static str, Map<&'static str, Map<&'static str, &'static str>>>,
    }

    let config = Config {
        block: indexmap! {
            "foo" => indexmap! {
                "one" => indexmap! { "a" => "b" },
                "two" => indexmap! { "c" => "d" },
            },
        },
    };

    let expected = indoc! {r#"
        block "foo" "one" {
          a = "b"
        }

        block "foo" "two" {
          c = "d"
        }
    "#};

    assert_serialize(config, expected);
}

#[test]
fn custom_block_structs() {
    #[derive(Serialize)]
    struct A {
        b: &'static str,
        #[serde(serialize_with = "block")]
        d: Vec<D>,
    }

    #[derive(Serialize)]
    struct B {
        #[serde(serialize_with = "labeled_block")]
        c1: C,
        #[serde(serialize_with = "labeled_block")]
        c2: Vec<C>,
    }

    #[derive(Serialize)]
    struct C {
        d: D,
    }

    #[derive(Serialize)]
    struct D {
        e: &'static str,
    }

    #[derive(Serialize)]
    struct Config {
        #[serde(serialize_with = "block")]
        a: A,
        #[serde(serialize_with = "labeled_block")]
        b: B,
    }

    let config = Config {
        a: A {
            b: "c",
            d: vec![D { e: "f1" }, D { e: "f2" }],
        },
        b: B {
            c1: C { d: D { e: "f3" } },
            c2: vec![C { d: D { e: "f4" } }, C { d: D { e: "f5" } }],
        },
    };

    let expected = indoc! {r#"
        a {
          b = "c"

          d {
            e = "f1"
          }

          d {
            e = "f2"
          }
        }

        b "c1" "d" {
          e = "f3"
        }

        b "c2" "d" {
          e = "f4"
        }

        b "c2" "d" {
          e = "f5"
        }
    "#};

    assert_serialize(config, expected);
}

#[test]
fn custom_terraform_blocks() {
    type Body = Map<&'static str, Value>;
    type LabeledBlock = Map<&'static str, Body>;

    #[derive(Serialize)]
    #[serde(untagged)]
    enum Value {
        Expression(Expression),
        #[serde(serialize_with = "block")]
        Block(Vec<Body>),
        #[serde(serialize_with = "labeled_block")]
        LabeledBlock(LabeledBlock),
    }

    #[derive(Serialize)]
    struct Config {
        #[serde(rename = "resource", serialize_with = "doubly_labeled_block")]
        resources: Map<&'static str, LabeledBlock>,
    }

    let config = Config {
        resources: indexmap! {
            "aws_s3_bucket" => indexmap! {
                "mybucket" => indexmap! {
                    "bucket" => Value::Expression("mybucket".into()),
                    "force_destroy" => Value::Expression(true.into()),
                    "server_side_encryption_configuration" => Value::Block(vec![
                        indexmap! {
                            "rule" => Value::Block(vec![
                                indexmap! {
                                    "apply_server_side_encryption_by_default" => Value::Block(vec![
                                        indexmap! {
                                            "kms_master_key_id" => Value::Expression(
                                                Traversal::builder(Variable::new("aws_kms_key").unwrap())
                                                    .attr("mykey")
                                                    .attr("arn")
                                                    .build()
                                                    .into(),
                                            ),
                                            "sse_algorithm" => Value::Expression("aws:kms".into()),
                                        }
                                    ]),
                                },
                            ]),
                        },
                    ]),
                    "dynamic" => Value::LabeledBlock(indexmap! {
                        "kubernetes_network_config" => indexmap! {
                            "for_each" => Value::Expression(
                                Conditional::new(
                                    BinaryOp::new(
                                        Traversal::builder(Variable::new("var").unwrap())
                                            .attr("service_ipv4_cidr")
                                            .build(),
                                        BinaryOperator::Eq,
                                        Expression::Null,
                                    ),
                                    Expression::Array(vec![]),
                                    Expression::from_iter([
                                        Traversal::builder(Variable::new("var").unwrap())
                                            .attr("service_ipv4_cidr")
                                            .build(),
                                    ]),
                                )
                                .into(),
                            ),
                            "content" => Value::Block(vec![
                                indexmap! {
                                    "service_ipv4_cidr" => Value::Expression(
                                        Traversal::builder(Variable::new("kubernetes_network_config").unwrap())
                                            .attr("value")
                                            .build()
                                            .into(),
                                    ),
                                },
                            ]),
                        },
                    }),
                    "tags" => Value::Expression(
                        Expression::from_iter([("application", "myapp"), ("environment", "dev")]),
                    ),
                },
            },
        },
    };

    let expected = indoc! {r#"
        resource "aws_s3_bucket" "mybucket" {
          bucket = "mybucket"
          force_destroy = true

          server_side_encryption_configuration {
            rule {
              apply_server_side_encryption_by_default {
                kms_master_key_id = aws_kms_key.mykey.arn
                sse_algorithm = "aws:kms"
              }
            }
          }

          dynamic "kubernetes_network_config" {
            for_each = var.service_ipv4_cidr == null ? [] : [var.service_ipv4_cidr]

            content {
              service_ipv4_cidr = kubernetes_network_config.value
            }
          }

          tags = {
            "application" = "myapp"
            "environment" = "dev"
          }
        }
    "#};

    assert_serialize(config, expected);
}
