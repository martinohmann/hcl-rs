mod common;

use common::assert_deserialize;
use hcl::expr::{
    BinaryOp, BinaryOperator, Expression, ForExpr, FuncCall, Heredoc, HeredocStripMode, ObjectKey,
    Operation, TemplateExpr, Traversal, UnaryOp, UnaryOperator, Variable,
};
use hcl::structure::{Block, Body};
use hcl::{Identifier, Value};
use indoc::indoc;
use serde::Deserialize;
use std::fmt::Debug;

#[test]
fn simple() {
    assert_deserialize(r#"foo = "bar""#, hcl::value!({ foo = "bar" }))
}

#[test]
fn array() {
    let input = r#"foo = [true, 2, "three", var.enabled]"#;
    let expected = hcl::value!({ foo = [true, 2, "three", "${var.enabled}"] });
    assert_deserialize(input, expected);
}

#[test]
fn object() {
    assert_deserialize(
        r#"foo = { bar = 42, "baz" = true }"#,
        hcl::value!({ foo = { bar = 42, baz = true } }),
    );

    assert_deserialize(
        indoc! {r#"
            foo = {
                bar = 42
                "baz" = true
            }
        "#},
        hcl::value!({ foo = { bar = 42, baz = true } }),
    );
}

#[test]
fn custom_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        foo: u32,
    }

    assert_deserialize(r#"foo = 1"#, Test { foo: 1 });
}

#[test]
fn custom_enum() {
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

    assert_deserialize(r#"value = "Unit""#, Test { value: E::Unit });
    assert_deserialize(r#"Newtype = 1"#, E::Newtype(1));
    assert_deserialize(r#"Tuple = [1,2]"#, E::Tuple(1, 2));
    assert_deserialize(
        r#"value = {"Struct" = {"a" = 1}}"#,
        Test {
            value: E::Struct { a: 1 },
        },
    );
}

#[test]
fn block() {
    assert_deserialize(
        r#"resource "aws_s3_bucket" "mybucket" { name = "mybucket" }"#,
        hcl::value!({ resource = { aws_s3_bucket = { mybucket = { name = "mybucket" } } } }),
    );

    assert_deserialize(
        r#"block { name = "asdf" }"#,
        hcl::value!({ block = { name = "asdf" } }),
    );
}

#[test]
fn duplicate_block() {
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

    let expected = hcl::value!({
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
    assert_deserialize(input, expected);

    let input = r#"
        foo { bar = "baz" }
        foo { bar = 1 }
    "#;
    let expected = hcl::value!({ foo = [{ bar = "baz" }, { bar = 1 }] });
    assert_deserialize(input, expected);
}

#[test]
fn duplicate_attribute() {
    let input = r#"
        foo = ["bar"]
        foo = ["baz"]
    "#;
    assert_deserialize(input, hcl::value!({ foo = ["baz"] }));
}

#[test]
fn duplicate_attribute_and_block() {
    let input = r#"
        foo = ["bar"]
        foo { bar = "baz" }
    "#;
    assert_deserialize(input, hcl::value!({ foo = { bar = "baz" } }));

    let input = r#"
        foo { bar = "baz" }
        foo = ["bar"]
    "#;
    assert_deserialize(input, hcl::value!({ foo = ["bar"] }));
}

#[test]
fn func_call() {
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

    assert_deserialize(input, expected);
}

#[test]
fn operations() {
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

    assert_deserialize(input, expected);
}

#[test]
fn for_exprs() {
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

    assert_deserialize(input, expected);
}

#[test]
fn negative_numbers() {
    let input = r#"
        float = -4.2
        float_exp = -4.2e10
        signed = -42
    "#;

    let expected = Body::builder()
        .add_attribute(("float", -4.2f64))
        .add_attribute(("float_exp", -4.2e10f64))
        .add_attribute(("signed", -42))
        .build();

    assert_deserialize(input, expected);
}

#[test]
#[cfg_attr(feature = "pest", ignore)]
fn template_expr() {
    let input = r#"foo = "bar ${baz} %{~ if cond}qux%{ endif ~}""#;

    let expected = Body::builder()
        .add_attribute((
            "foo",
            TemplateExpr::QuotedString("bar ${baz} %{~ if cond }qux%{ endif ~}".into()),
        ))
        .build();

    assert_deserialize(input, expected);
}

#[test]
fn null_in_variable_expr() {
    let input = "foo = null_foo";

    let expected = Body::builder()
        .add_attribute(("foo", Variable::unchecked("null_foo")))
        .build();

    assert_deserialize(input, expected);
}

#[test]
fn object_with_variable_expr_key() {
    let input = "providers = { aws.eu-central-1 = aws.eu-central-1 }";

    let expected = Body::builder()
        .add_attribute((
            "providers",
            Expression::from_iter([(
                ObjectKey::Expression(Expression::from(Traversal::new(
                    Variable::unchecked("aws"),
                    [Identifier::unchecked("eu-central-1")],
                ))),
                Expression::from(Traversal::new(
                    Variable::unchecked("aws"),
                    [Identifier::unchecked("eu-central-1")],
                )),
            )]),
        ))
        .build();

    assert_deserialize(input, expected);
}

#[test]
fn traversal_with_expression() {
    let input =
        "route_table_id = aws_route_table.private[count.index % var.availability_zone_count].id";

    let expected = Body::builder()
        .add_attribute((
            "route_table_id",
            Traversal::builder(Variable::unchecked("aws_route_table"))
                .attr("private")
                .index(BinaryOp::new(
                    Traversal::builder(Variable::unchecked("count"))
                        .attr("index")
                        .build(),
                    BinaryOperator::Mod,
                    Traversal::builder(Variable::unchecked("var"))
                        .attr("availability_zone_count")
                        .build(),
                ))
                .attr("id")
                .build(),
        ))
        .build();

    assert_deserialize(input, expected);
}

#[test]
#[cfg_attr(feature = "pest", ignore)]
fn unescape_strings() {
    let input = r#"
        block "label\\with\\backslashes" {
          string_attr = "I \u2665 unicode"

          object_attr = {
            "key\nwith\nnewlines" = true
          }

          heredoc = <<-EOS
            heredoc template with \
            escaped newline and \\backslash is not unescaped yet
          EOS
        }
    "#;

    let expected = Body::builder()
        .add_block(
            Block::builder("block")
                .add_label("label\\with\\backslashes")
                .add_attribute(("string_attr", "I \u{2665} unicode"))
                .add_attribute((
                    "object_attr",
                    Expression::from_iter([("key\nwith\nnewlines", true)]),
                ))
                .add_attribute((
                    "heredoc",
                    TemplateExpr::Heredoc(
                        Heredoc::new(
                            Identifier::unchecked("EOS"),
                            "            heredoc template with escaped newline and \\backslash is not unescaped yet\n"
                        )
                        .with_strip_mode(HeredocStripMode::Indent)
                    )
                ))
                .build(),
        )
        .build();

    assert_deserialize(input, expected);
}

#[test]
fn errors() {
    assert!(hcl::from_str::<Value>(r#"invalid["#).is_err());
}

#[test]
fn terraform() {
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
        }
    "#;

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

    assert_deserialize(input, expected);
}
