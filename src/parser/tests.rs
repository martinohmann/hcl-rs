use super::*;
use crate::template::{IfDirective, StripMode, Template};
use pest::*;
use pretty_assertions::assert_eq;

#[track_caller]
fn expect_body(input: &str, expected: Body) {
    assert_eq!(parse(input).unwrap(), expected);
}

#[test]
fn parse_identifier() {
    parses_to! {
        parser: HclParser,
        input: "_an-id3nt1fieR",
        rule: Rule::Identifier,
        tokens: [
            Identifier(0, 14)
        ]
    };
}

#[test]
fn parse_string() {
    parses_to! {
        parser: HclParser,
        input: "\"a string\"",
        rule: Rule::StringLit,
        tokens: [
            StringLit(0, 10, [
                String(1, 9)
            ])
        ]
    };
}

#[test]
fn parse_number() {
    parses_to! {
        parser: HclParser,
        input: "12e+10",
        rule: Rule::NumericLit,
        tokens: [
            Float(0, 6)
        ]
    };

    parses_to! {
        parser: HclParser,
        input: "42",
        rule: Rule::NumericLit,
        tokens: [
            Int(0, 2)
        ]
    };
}

#[test]
fn parse_conditional() {
    parses_to! {
        parser: HclParser,
        input: "var.enabled ? 1 : 0",
        rule: Rule::Expression,
        tokens: [
            Expression(0, 19, [
                ExprTerm(0, 11, [
                    Variable(0, 3),
                    GetAttr(3, 11, [
                        Identifier(4, 11)
                    ])
                ]),
                Expression(14, 16, [
                    ExprTerm(14, 16, [
                        Int(14, 15)
                    ])
                ]),
                Expression(18, 19, [
                    ExprTerm(18, 19, [
                        Int(18, 19)
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_collections() {
    parses_to! {
        parser: HclParser,
        input: r#"foo = ["bar", ["baz"]]"#,
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 22, [
                Identifier(0, 3),
                Expression(6, 22, [
                    ExprTerm(6, 22, [
                        Tuple(6, 22, [
                            Expression(7, 12, [
                                ExprTerm(7, 12, [
                                    StringLit(7, 12, [
                                        String(8, 11)
                                    ])
                                ])
                            ]),
                            Expression(14, 21, [
                                ExprTerm(14, 21, [
                                    Tuple(14, 21, [
                                        Expression(15, 20, [
                                            ExprTerm(15, 20, [
                                                StringLit(15, 20, [
                                                    String(16, 19)
                                                ])
                                            ])
                                        ])
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };

    parses_to! {
        parser: HclParser,
        input: r#"foo = {"bar" = "baz","qux" = ident }"#,
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 36, [
                Identifier(0, 3),
                Expression(6, 36, [
                    ExprTerm(6, 36, [
                        Object(6, 36, [
                            Expression(7, 13, [
                                ExprTerm(7, 13, [
                                    StringLit(7, 12, [
                                        String(8, 11)
                                    ])
                                ])
                            ]),
                            Expression(15, 20, [
                                ExprTerm(15, 20, [
                                    StringLit(15, 20, [
                                        String(16, 19)
                                    ])
                                ])
                            ]),
                            Expression(21, 27, [
                                ExprTerm(21, 27, [
                                    StringLit(21, 26, [
                                        String(22, 25)
                                    ])
                                ])
                            ]),
                            Expression(29, 35, [
                                ExprTerm(29, 35, [
                                    Variable(29, 34)
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_template() {
    parses_to! {
        parser: HclParser,
        input: "<<HEREDOC\n${foo}\n%{if asdf}qux%{endif}\nheredoc\nHEREDOC",
        rule: Rule::ExprTerm,
        tokens: [
            ExprTerm(0, 54, [
                TemplateExpr(0, 54, [
                    HeredocTemplate(0, 54, [
                        HeredocIntroNormal(0, 2),
                        Identifier(2, 9),
                        HeredocContent(10, 47)
                    ])
                ])
            ])
        ]
    };

    parses_to! {
        parser: HclParser,
        input: r#""foo ${bar} $${baz}, %{if cond ~} qux %{~ endif}""#,
        rule: Rule::ExprTerm,
        tokens: [
            ExprTerm(0, 49, [
                TemplateExpr(0, 49, [
                    QuotedStringTemplate(0, 49, [
                        QuotedStringTemplateInner(1, 48, [
                            QuotedStringTemplateLiteral(1, 5),
                            TemplateInterpolation(5, 11, [
                                TemplateIExprStartNormal(5, 7),
                                Expression(7, 10, [
                                    ExprTerm(7, 10, [
                                        Variable(7, 10)
                                    ])
                                ]),
                                TemplateExprEndNormal(10, 11)
                            ]),
                            QuotedStringTemplateLiteral(11, 21),
                            TemplateDirective(21, 48, [
                                TemplateIf(21, 48, [
                                    TemplateIfExpr(21, 38, [
                                        TemplateDExprStartNormal(21, 23),
                                        Expression(26, 31, [
                                            ExprTerm(26, 31, [
                                                Variable(26, 30)
                                            ])
                                        ]),
                                        TemplateExprEndStrip(31, 33),
                                        Template(34, 38, [
                                            TemplateLiteral(34, 38)
                                        ]),
                                    ]),
                                    TemplateEndIfExpr(38, 48, [
                                        TemplateDExprStartStrip(38, 41),
                                        TemplateExprEndNormal(47, 48),
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_cond_in_interpolation() {
    parses_to! {
        parser: HclParser,
        input: r#"name = "${var.l ? "us-east-1." : ""}""#,
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 37, [
                Identifier(0, 4),
                Expression(7, 37, [
                    ExprTerm(7, 37, [
                        TemplateExpr(7, 37, [
                            QuotedStringTemplate(7, 37, [
                                QuotedStringTemplateInner(8, 36, [
                                    TemplateInterpolation(8, 36, [
                                        TemplateIExprStartNormal(8, 10),
                                        Expression(10, 35, [
                                            ExprTerm(10, 15, [
                                                Variable(10, 13),
                                                GetAttr(13, 15, [
                                                    Identifier(14, 15)
                                                ])
                                            ]),
                                            Expression(18, 31, [
                                                ExprTerm(18, 31, [
                                                    StringLit(18, 30, [
                                                        String(19, 29)
                                                    ])
                                                ])
                                            ]),
                                            Expression(33, 35, [
                                                ExprTerm(33, 35, [
                                                    StringLit(33, 35, [
                                                        String(34, 34)
                                                    ])
                                                ])
                                            ])
                                        ]),
                                        TemplateExprEndNormal(35, 36)
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_object_with_variable_expr_key() {
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

    expect_body(input, expected);
}

#[test]
fn parse_nested_function_call_with_splat() {
    parses_to! {
        parser: HclParser,
        input: r#"element(concat(aws_kms_key.key-one.*.arn, aws_kms_key.key-two.*.arn), 0)"#,
        rule: Rule::FunctionCall,
        tokens: [
            FunctionCall(0, 72, [
                Identifier(0, 7),
                Arguments(7, 72, [
                    Expression(8, 68, [
                        ExprTerm(8, 68, [
                            FunctionCall(8, 68, [
                                Identifier(8, 14),
                                Arguments(14, 68, [
                                    Expression(15, 40, [
                                        ExprTerm(15, 40, [
                                            Variable(15, 26),
                                            GetAttr(26, 34, [
                                                Identifier(27, 34)
                                            ]),
                                            AttrSplat(34, 36),
                                            GetAttr(36, 40, [
                                                Identifier(37, 40)
                                            ]),
                                        ])
                                    ]),
                                    Expression(42, 67, [
                                        ExprTerm(42, 67, [
                                            Variable(42, 53),
                                            GetAttr(53, 61, [
                                                Identifier(54, 61)
                                            ]),
                                            AttrSplat(61, 63),
                                            GetAttr(63, 67, [
                                                Identifier(64, 67)
                                            ])
                                        ])
                                    ])
                                ])
                            ])
                        ])
                    ]),
                    Expression(70, 71, [
                        ExprTerm(70, 71, [
                            Int(70, 71)
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_traversal_with_expression() {
    let input =
        "route_table_id = aws_route_table.private[count.index % var.availability_zone_count].id";

    let expected = Body::builder()
        .add_attribute((
            "route_table_id",
            Expression::from(Traversal::new(
                Variable::unchecked("aws_route_table"),
                [
                    TraversalOperator::GetAttr("private".into()),
                    TraversalOperator::Index(Expression::from(Operation::Binary(BinaryOp::new(
                        Traversal::new(
                            Variable::unchecked("count"),
                            [TraversalOperator::GetAttr("index".into())],
                        ),
                        BinaryOperator::Mod,
                        Traversal::new(
                            Variable::unchecked("var"),
                            [TraversalOperator::GetAttr("availability_zone_count".into())],
                        ),
                    )))),
                    TraversalOperator::GetAttr("id".into()),
                ],
            )),
        ))
        .build();

    expect_body(input, expected);
}

#[test]
fn parse_null_in_variable_expr() {
    let input = "foo = null_foo";

    let expected = Body::builder()
        .add_attribute(("foo", Variable::unchecked("null_foo")))
        .build();

    expect_body(input, expected);
}

#[test]
fn parse_escaped_slash_in_string() {
    parses_to! {
        parser: HclParser,
        input: r#""\\""#,
        rule: Rule::StringLit,
        tokens: [
            StringLit(0, 4, [
                String(1, 3),
            ])
        ]
    };
}

#[test]
fn parse_hcl() {
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
        }"#;

    let expected = Body::builder()
        .add_block(
            Block::builder("resource")
                .add_label("aws_s3_bucket")
                .add_label("mybucket")
                .add_attribute(Attribute::new("bucket", "mybucket"))
                .add_attribute(Attribute::new("force_destroy", true))
                .add_block(
                    Block::builder("server_side_encryption_configuration")
                        .add_block(
                            Block::builder("rule")
                                .add_block(
                                    Block::builder("apply_server_side_encryption_by_default")
                                        .add_attribute(Attribute::new(
                                            "kms_master_key_id",
                                            Traversal::new(
                                                Variable::unchecked("aws_kms_key"),
                                                ["mykey", "arn"],
                                            ),
                                        ))
                                        .add_attribute(Attribute::new("sse_algorithm", "aws:kms"))
                                        .build(),
                                )
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .build();

    expect_body(input, expected);
}

#[test]
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
                            "            heredoc template with \\\n            escaped newline and \\\\backslash is not unescaped yet\n"
                        )
                        .with_strip_mode(HeredocStripMode::Indent)
                    )
                ))
                .build(),
        )
        .build();

    expect_body(input, expected);
}

#[test]
fn parse_negative_numbers() {
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

    expect_body(input, expected);
}

#[test]
fn parse_template_expr() {
    let input = r#"foo = "bar ${baz} %{~ if cond}qux%{ endif ~}""#;

    let expected = Body::builder()
        .add_attribute((
            "foo",
            TemplateExpr::QuotedString("bar ${baz} %{~ if cond}qux%{ endif ~}".into()),
        ))
        .build();

    let body = parse(input).unwrap();

    assert_eq!(body, expected);

    match body.attributes().next().unwrap().expr() {
        Expression::TemplateExpr(expr) => {
            let template = Template::from_expr(expr).unwrap();

            let expected_template = Template::new()
                .add_literal("bar ")
                .add_interpolation(Variable::unchecked("baz"))
                .add_literal(" ")
                .add_directive(
                    IfDirective::new(
                        Variable::unchecked("cond"),
                        Template::new().add_literal("qux"),
                    )
                    .with_if_strip(StripMode::Start)
                    .with_endif_strip(StripMode::End),
                );

            assert_eq!(template, expected_template);
        }
        _ => unreachable!(),
    }
}
