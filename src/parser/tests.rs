use super::*;
use pest::*;
use pretty_assertions::assert_eq;

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
fn parse_attr() {
    parses_to! {
        parser: HclParser,
        input: "foo = \"bar\"",
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 11, [
                Identifier(0, 3),
                Expression(6, 11, [
                    ExprTerm(6, 11, [
                        StringLit(6, 11, [
                            String(7, 10)
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn conditional() {
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
fn parse_terraform() {
    parses_to! {
        parser: HclParser,
        input: r#"
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
}
            "#,
        rule: Rule::Hcl,
        tokens: [
            Body(1, 299, [
                Block(1, 299, [
                    Identifier(1, 9),
                    StringLit(10, 25, [
                        String(11, 24)
                    ]),
                    StringLit(26, 36, [
                        String(27, 35)
                    ]),
                    BlockBody(37, 299, [
                        Body(41, 297, [
                            Attribute(41, 70, [
                                Identifier(41, 47),
                                Expression(57, 70, [
                                    ExprTerm(57, 70, [
                                        StringLit(57, 67, [
                                            String(58, 66)
                                        ])
                                    ])
                                ])
                            ]),
                            Attribute(70, 94, [
                                Identifier(70, 83),
                                Expression(86, 94, [
                                    ExprTerm(86, 94, [
                                        BooleanLit(86, 90)
                                    ])
                                ])
                            ]),
                            Block(94, 297, [
                                Identifier(94, 130),
                                BlockBody(131, 297, [
                                    Body(137, 293, [
                                        Block(137, 293, [
                                            Identifier(137, 141),
                                            BlockBody(142, 293, [
                                                Body(150, 287, [
                                                    Block(150, 287, [
                                                        Identifier(150, 189),
                                                        BlockBody(190, 287, [
                                                            Body(200, 286, [
                                                                Attribute(200, 250, [
                                                                    Identifier(200, 217),
                                                                    Expression(220, 250, [
                                                                        ExprTerm(220, 241, [
                                                                            Variable(220, 231),
                                                                            GetAttr(231, 237, [
                                                                                Identifier(232, 237)
                                                                            ]),
                                                                            GetAttr(237, 241, [
                                                                                Identifier(238, 241)
                                                                            ])
                                                                        ])
                                                                    ])
                                                                ]),
                                                                Attribute(250, 286, [
                                                                    Identifier(250, 263),
                                                                    Expression(270, 286, [
                                                                        ExprTerm(270, 286, [
                                                                            StringLit(270, 279, [
                                                                                String(271, 278)
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
    parses_to! {
        parser: HclParser,
        input: r#"
providers = {
  aws.eu-central-1 = aws.eu-central-1
  aws.eu-west-1    = aws.eu-west-1
}"#,
        rule: Rule::Hcl,
        tokens: [
            Body(1, 89, [
                Attribute(1, 89, [
                    Identifier(1, 10),
                    Expression(13, 89, [
                        ExprTerm(13, 89, [
                            Object(13, 89, [
                                Expression(17, 34, [
                                    ExprTerm(17, 33, [
                                        Variable(17, 20),
                                        GetAttr(20, 33, [
                                            Identifier(21, 33)
                                        ]),
                                    ])
                                ]),
                                Expression(36, 55, [
                                    ExprTerm(36, 52, [
                                        Variable(36, 39),
                                        GetAttr(39, 52, [
                                            Identifier(40, 52)
                                        ]),
                                    ])
                                ]),
                                Expression(55, 72, [
                                    ExprTerm(55, 68, [
                                        Variable(55, 58),
                                        GetAttr(58, 68, [
                                            Identifier(59, 68)
                                        ]),
                                    ])
                                ]),
                                Expression(74, 88, [
                                    ExprTerm(74, 87, [
                                        Variable(74, 77),
                                        GetAttr(77, 87, [
                                            Identifier(78, 87)
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
    parses_to! {
        parser: HclParser,
        input: r#"route_table_id = aws_route_table.private[count.index % var.availability_zone_count].id"#,
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 86, [
                Identifier(0, 14),
                Expression(17, 86, [
                    ExprTerm(17, 86, [
                        Variable(17, 32),
                        GetAttr(32, 40, [
                            Identifier(33, 40)
                        ]),
                        Index(40, 83, [
                            Expression(41, 82, [
                                ExprTerm(41, 52, [
                                    Variable(41, 46),
                                    GetAttr(46, 52, [
                                        Identifier(47, 52)
                                    ]),
                                ]),
                                BinaryOperator(53, 54, [
                                    ArithmeticOperator(53, 54)
                                ]),
                                Expression(55, 82, [
                                    ExprTerm(55, 82, [
                                        Variable(55, 58),
                                        GetAttr(58, 82, [
                                            Identifier(59, 82)
                                        ]),
                                    ])
                                ])
                            ])
                        ]),
                        GetAttr(83, 86, [
                            Identifier(84, 86)
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parse_null_in_variable_expr() {
    parses_to! {
        parser: HclParser,
        input: r#"foo = null_foo"#,
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 14, [
                Identifier(0, 3),
                Expression(6, 14, [
                    ExprTerm(6, 14, [
                        Variable(6, 14)
                    ])
                ])
            ])
        ]
    };
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

    let body = parse(input).unwrap();

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
                                                Identifier::new("aws_kms_key"),
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

    assert_eq!(body, expected);
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

    let body = parse(input).unwrap();

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
                            Identifier::new("EOS"),
                            "            heredoc template with \\\n            escaped newline and \\\\backslash is not unescaped yet\n"
                        )
                        .with_strip_mode(HeredocStripMode::Indent)
                    )
                ))
                .build(),
        )
        .build();

    assert_eq!(body, expected);
}

#[test]
fn negative_numbers() {
    let input = r#"
        float = -4.2
        float_exp = -4.2e10
        signed = -42
    "#;

    let body = parse(input).unwrap();

    let expected = Body::builder()
        .add_attribute(("float", -4.2f64))
        .add_attribute(("float_exp", -4.2e10f64))
        .add_attribute(("signed", -42))
        .build();

    assert_eq!(body, expected);
}

#[test]
fn template_expr() {
    use crate::{
        structure::TemplateExpr,
        template::{IfDirective, StripMode, Template},
    };

    let input = r#"foo = "bar ${baz} %{~ if cond}qux%{ endif ~}""#;
    let body = parse(input).unwrap();

    let expected = Body::builder()
        .add_attribute((
            "foo",
            TemplateExpr::QuotedString("bar ${baz} %{~ if cond}qux%{ endif ~}".into()),
        ))
        .build();

    assert_eq!(body, expected);

    match body.attributes().next().unwrap().expr() {
        Expression::TemplateExpr(expr) => {
            let template = Template::from_expr(expr).unwrap();

            let expected_template = Template::new()
                .add_literal("bar ")
                .add_interpolation(Expression::Variable(Identifier::new("baz")))
                .add_literal(" ")
                .add_directive(
                    IfDirective::new(
                        Expression::Variable(Identifier::new("cond")),
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
