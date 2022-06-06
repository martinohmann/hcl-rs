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
        input: "-12e+10",
        rule: Rule::NumericLit,
        tokens: [
            Float(0, 7)
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
                StringLit(6, 11, [
                    String(7, 10)
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
        rule: Rule::Conditional,
        tokens: [
            Conditional(0, 19, [
                CondExpr(0, 11, [
                    VariableExpr(0, 11)
                ]),
                Int(14, 15),
                Int(18, 19)
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
                            Attribute(41, 67, [
                                Identifier(41, 47),
                                StringLit(57, 67, [
                                    String(58, 66)
                                ])
                            ]),
                            Attribute(70, 90, [
                                Identifier(70, 83),
                                BooleanLit(86, 90)
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
                                                            Body(200, 279, [
                                                                Attribute(200, 241, [
                                                                    Identifier(200, 217),
                                                                    VariableExpr(220, 241)
                                                                ]),
                                                                Attribute(250, 279, [
                                                                    Identifier(250, 263),
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
                Tuple(6, 22, [
                    StringLit(7, 12, [
                        String(8, 11)
                    ]),
                    Tuple(14, 21, [
                        StringLit(15, 20, [
                            String(16, 19)
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
                Object(6, 36, [
                    StringLit(7, 12, [
                        String(8, 11)
                    ]),
                    StringLit(15, 20, [
                        String(16, 19)
                    ]),
                    StringLit(21, 26, [
                        String(22, 25)
                    ]),
                    VariableExpr(29, 34)
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
            Heredoc(0, 54, [
                HeredocIntroNormal(0, 2),
                Identifier(2, 9),
                HeredocContent(10, 47)
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
                StringLit(7, 37, [
                    String(8, 36, [
                        TemplateInterpolation(8, 36, [
                            Conditional(10, 35, [
                                CondExpr(10, 15, [
                                    VariableExpr(10, 15)
                                ]),
                                StringLit(18, 30, [
                                    String(19, 29)
                                ]),
                                StringLit(33, 35, [
                                    String(34, 34)
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
}
                "#,
        rule: Rule::Hcl,
        tokens: [
            Body(1, 89, [
                Attribute(1, 89, [
                    Identifier(1, 10),
                    Object(13, 89, [
                        VariableExpr(17, 33),
                        VariableExpr(36, 52),
                        VariableExpr(55, 68),
                        VariableExpr(74, 87)
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
                Arguments(8, 71, [
                    FunctionCall(8, 68, [
                        Arguments(15, 67, [
                            VariableExpr(15, 40),
                            VariableExpr(42, 67)
                        ])
                    ]),
                    Int(70, 71)
                ])
            ])
        ]
    };
}

#[test]
fn parse_element_access_with_expression() {
    parses_to! {
        parser: HclParser,
        input: r#"route_table_id = aws_route_table.private[count.index % var.availability_zone_count].id"#,
        rule: Rule::Attribute,
        tokens: [
            Attribute(0, 86, [
                Identifier(0, 14),
                VariableExpr(17, 86, [
                    IndexExpr(41, 82, [
                        Operation(41, 82, [
                            BinaryOp(41, 82, [
                                VariableExpr(41, 52),
                                BinaryOperator(53, 54, [
                                    ArithmeticOperator(53, 54)
                                ]),
                                VariableExpr(55, 82)
                            ])
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
                VariableExpr(6, 14)
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
                                            RawExpression::new("aws_kms_key.mykey.arn"),
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
                .build(),
        )
        .build();

    assert_eq!(body, expected);
}
