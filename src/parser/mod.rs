mod ast;

pub use ast::{interpolate, Node};

use crate::{Error, Result};
use pest::Parser as ParserTrait;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar/hcl.pest"]
pub(crate) struct HclParser;

pub(crate) fn parse(input: &str) -> Result<ast::Node<'_>> {
    let pair = HclParser::parse(Rule::hcl, input)
        .map_err(|e| Error::ParseError(e.to_string()))?
        .next()
        .unwrap();
    Ok(Node::from_pair(pair))
}

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
                    identifier(1, 9),
                    block_labeled(10, 299, [
                        string_lit(10, 25, [
                            string(11, 24)
                        ]),
                        block_labeled(26, 299, [
                            string_lit(26, 36, [
                                string(27, 35)
                            ]),
                            block_body(37, 299, [
                                block_body_inner(41, 297, [
                                    attribute(41, 67, [
                                        identifier(41, 47),
                                        string_lit(57, 67, [
                                            string(58, 66)
                                        ])
                                    ]),
                                    attribute(70, 90, [
                                        identifier(70, 83),
                                        boolean_lit(86, 90)
                                    ]),
                                    block(94, 297, [
                                        identifier(94, 130),
                                        block_body(131, 297, [
                                            block_body_inner(137, 293, [
                                                block(137, 293, [
                                                    identifier(137, 141),
                                                    block_body(142, 293, [
                                                        block_body_inner(150, 287, [
                                                            block(150, 287, [
                                                                identifier(150, 189),
                                                                block_body(190, 287, [
                                                                    block_body_inner(200, 279, [
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

    #[test]
    fn object_with_variable_expr_key() {
        parses_to! {
            parser: HclParser,
            input: r#"
providers = {
  aws.eu-central-1 = aws.eu-central-1
  aws.eu-west-1    = aws.eu-west-1
}
                "#,
            rule: Rule::config_file,
            tokens: [
                config_file(0, 89, [
                    attribute(1, 89, [
                        identifier(1, 10),
                        object(13, 89, [
                            variable_expr(17, 33),
                            variable_expr(36, 52),
                            variable_expr(55, 68),
                            variable_expr(74, 87)
                        ])
                    ])
                ])
            ]
        };
    }

    #[test]
    fn nested_function_call_with_splat() {
        parses_to! {
            parser: HclParser,
            input: r#"element(concat(aws_kms_key.key-one.*.arn, aws_kms_key.key-two.*.arn), 0)"#,
            rule: Rule::function_call,
            tokens: [
                function_call(0, 72, [
                    arguments(8, 71, [
                        function_call(8, 68, [
                            arguments(15, 67, [
                                variable_expr(15, 40),
                                variable_expr(42, 67)
                            ])
                        ]),
                        int(70, 71)
                    ])
                ])
            ]
        };
    }

    #[test]
    fn element_access_with_expression() {
        parses_to! {
            parser: HclParser,
            input: r#"route_table_id = aws_route_table.private[count.index % var.availability_zone_count].id"#,
            rule: Rule::attribute,
            tokens: [
                attribute(0, 86, [
                    identifier(0, 14),
                    variable_expr(17, 86, [
                        index_expression(41, 82, [
                            operation(41, 82, [
                                binary_op(41, 82, [
                                    variable_expr(41, 52),
                                    binary_operator(53, 54, [
                                        arithmetic_operator(53, 54)
                                    ]),
                                    variable_expr(55, 82)
                                ])
                            ])
                        ])
                    ])
                ])
            ]
        };
    }

    #[test]
    fn null_in_variable_expr() {
        parses_to! {
            parser: HclParser,
            input: r#"foo = null_foo"#,
            rule: Rule::attribute,
            tokens: [
                attribute(0, 14, [
                    identifier(0, 3),
                    variable_expr(6, 14)
                ])
            ]
        };
    }

    #[test]
    fn escaped_slash_in_string() {
        parses_to! {
            parser: HclParser,
            input: r#""\\""#,
            rule: Rule::string_lit,
            tokens: [
                string_lit(0, 4, [
                    string(1, 3),
                ])
            ]
        };
    }
}
