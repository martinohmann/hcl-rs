use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/hcl.pest"]
pub(crate) struct HclParser;

#[cfg(test)]
mod test {
    use super::*;
    use pest::*;

    #[test]
    fn identifier() {
        parses_to! {
            parser: HclParser,
            input: "_an-id3nt1fieR",
            rule: Rule::identifier,
            tokens: [
                identifier(0, 14)
            ]
        };
    }

    #[test]
    fn string() {
        parses_to! {
            parser: HclParser,
            input: "\"a string\"",
            rule: Rule::string_lit,
            tokens: [
                string_lit(0, 10, [
                    string(1, 9)
                ])
            ]
        };
    }

    #[test]
    fn number() {
        parses_to! {
            parser: HclParser,
            input: "-12e+10",
            rule: Rule::numeric_lit,
            tokens: [
                float(0, 7)
            ]
        };

        parses_to! {
            parser: HclParser,
            input: "42",
            rule: Rule::numeric_lit,
            tokens: [
                int(0, 2)
            ]
        };
    }

    #[test]
    fn attr() {
        parses_to! {
            parser: HclParser,
            input: "foo = \"bar\"",
            rule: Rule::attribute,
            tokens: [
                attribute(0, 11, [
                    identifier(0, 3),
                    string_lit(6, 11, [
                        string(7, 10)
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
            rule: Rule::conditional,
            tokens: [
                conditional(0, 19, [
                    cond_expr(0, 11, [
                        variable_expr(0, 11)
                    ]),
                    int(14, 15),
                    int(18, 19)
                ])
            ]
        };
    }

    #[test]
    fn terraform() {
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
            rule: Rule::body,
            tokens: [
                block(1, 299, [
                    block_identifier(1, 9, [
                        identifier(1, 9),
                    ]),
                    block_keys(10, 36, [
                        string_lit(10, 25, [
                            string(11, 24)
                        ]),
                        string_lit(26, 36, [
                            string(27, 35)
                        ])
                    ]),
                    block_body(41, 297, [
                        attribute(41, 67, [
                            identifier(41, 47),
                            string_lit(57, 67, [
                                string(58, 66)
                            ])
                        ]),
                        attribute(70, 90, [
                            identifier(70, 83),
                            boolean(86, 90)
                        ]),
                        block(94, 297, [
                            block_identifier(94, 130, [
                                identifier(94, 130)
                            ]),
                            block_keys(131, 131),
                            block_body(137, 293, [
                                block(137, 293, [
                                    block_identifier(137, 141, [
                                        identifier(137, 141)
                                    ]),
                                    block_keys(142, 142),
                                    block_body(150, 287, [
                                        block(150, 287, [
                                            block_identifier(150, 189, [
                                                identifier(150, 189)
                                            ]),
                                            block_keys(190, 190),
                                            block_body(200, 279, [
                                                attribute(200, 241, [
                                                    identifier(200, 217),
                                                    variable_expr(220, 241)
                                                ]),
                                                attribute(250, 279, [
                                                    identifier(250, 263),
                                                    string_lit(270, 279, [
                                                        string(271, 278)
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
    fn collections() {
        parses_to! {
            parser: HclParser,
            input: r#"foo = ["bar", ["baz"]]"#,
            rule: Rule::attribute,
            tokens: [
                attribute(0, 22, [
                    identifier(0, 3),
                    tuple(6, 22, [
                        string_lit(7, 12, [
                            string(8, 11)
                        ]),
                        tuple(14, 21, [
                            string_lit(15, 20, [
                                string(16, 19)
                            ])
                        ])
                    ])
                ])
            ]
        };

        parses_to! {
            parser: HclParser,
            input: r#"foo = {"bar" = "baz","qux" = ident }"#,
            rule: Rule::attribute,
            tokens: [
                attribute(0, 36, [
                    identifier(0, 3),
                    object(6, 36, [
                        string_lit(7, 12, [
                            string(8, 11)
                        ]),
                        string_lit(15, 20, [
                            string(16, 19)
                        ]),
                        string_lit(21, 26, [
                            string(22, 25)
                        ]),
                        variable_expr(29, 34)
                    ])
                ])
            ]
        };
    }

    #[test]
    fn template() {
        parses_to! {
            parser: HclParser,
            input: "<<HEREDOC\n${foo}\n%{if asdf}qux%{endif}\nheredoc\nHEREDOC",
            rule: Rule::expr_term,
            tokens: [
                heredoc(0, 54, [
                    identifier(2, 9),
                    template(10, 46)
                ])
            ]
        };
    }

    #[test]
    fn cond_in_interpolation() {
        parses_to! {
            parser: HclParser,
            input: r#"name = "${var.l ? "us-east-1." : ""}""#,
            rule: Rule::attribute,
            tokens: [
                attribute(0, 37, [
                    identifier(0, 4),
                    string_lit(7, 37, [
                        string(8, 36, [
                            template_interpolation(8, 36, [
                                conditional(10, 35, [
                                    cond_expr(10, 15, [
                                        variable_expr(10, 15)
                                    ]),
                                    string_lit(18, 30, [
                                        string(19, 29)
                                    ]),
                                    string_lit(33, 35, [
                                        string(34, 34)
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ]
        };
    }
}
