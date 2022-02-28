use crate::{
    structure::{Attribute, Block, BlockLabel, Body, Structure},
    Map, Result, Value,
};
use pest::{
    iterators::{Pair, Pairs},
    Parser as ParserTrait,
};
use pest_derive::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "parser/grammar/hcl.pest"]
struct HclParser;

/// Parses a HCL `Body` from a `&str`.
///
/// If deserialization into a different type is preferred consider using [`hcl::from_str`][from_str].
///
/// [from_str]: ./de/fn.from_str.html
///
/// ## Example
///
/// ```
/// use hcl::{Attribute, Block, Body};
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let input = r#"
///     some_attr = "foo"
///
///     some_block "some_block_label" {
///       attr = "value"
///     }
/// "#;
///
/// let expected = Body::builder()
///     .add_attribute(("some_attr", "foo"))
///     .add_block(
///         Block::builder("some_block")
///             .add_label("some_block_label")
///             .add_attribute(("attr", "value"))
///             .build()
///     )
///     .build();
///
/// let body = hcl::parse(input)?;
///
/// assert_eq!(body, expected);
/// #   Ok(())
/// # }
/// ```
pub fn parse(input: &str) -> Result<Body> {
    let pair = HclParser::parse(Rule::Hcl, input)?.next().unwrap();

    Ok(parse_body(pair))
}

fn parse_body(pair: Pair<Rule>) -> Body {
    pair.into_inner().map(parse_structure).collect()
}

fn parse_structure(pair: Pair<Rule>) -> Structure {
    match pair.as_rule() {
        Rule::Attribute => Structure::Attribute(parse_attribute(pair)),
        Rule::Block => Structure::Block(parse_block(pair)),
        rule => unexpected_rule(rule),
    }
}

fn parse_attribute(pair: Pair<Rule>) -> Attribute {
    let mut pairs = pair.into_inner();

    Attribute {
        key: parse_string(pairs.next().unwrap()),
        value: parse_value(pairs.next().unwrap()),
    }
}

fn parse_block(pair: Pair<Rule>) -> Block {
    let mut pairs = pair.into_inner();

    let identifier = parse_string(pairs.next().unwrap());

    let (labels, block_body): (Vec<Pair<Rule>>, Vec<Pair<Rule>>) =
        pairs.partition(|pair| pair.as_rule() != Rule::BlockBody);

    Block {
        identifier,
        labels: labels.into_iter().map(parse_block_label).collect(),
        body: parse_block_body(block_body.into_iter().next().unwrap()),
    }
}

fn parse_block_label(pair: Pair<Rule>) -> BlockLabel {
    match pair.as_rule() {
        Rule::Identifier => BlockLabel::Identifier(parse_string(pair)),
        Rule::StringLit => BlockLabel::StringLit(parse_string(inner(pair))),
        rule => unexpected_rule(rule),
    }
}

fn parse_block_body(pair: Pair<Rule>) -> Body {
    match pair.as_rule() {
        Rule::BlockBody => parse_body(inner(pair)),
        rule => unexpected_rule(rule),
    }
}

fn parse_value(pair: Pair<Rule>) -> Value {
    match pair.as_rule() {
        Rule::BooleanLit => Value::Bool(parse_primitive(pair)),
        Rule::Float => Value::Number(parse_primitive::<f64>(pair).into()),
        Rule::Heredoc => Value::String(parse_string(pair.into_inner().nth(1).unwrap())),
        Rule::Identifier => Value::String(parse_string(pair)),
        Rule::Int => Value::Number(parse_primitive::<i64>(pair).into()),
        Rule::NullLit => Value::Null,
        Rule::StringLit => Value::String(parse_string(inner(pair))),
        Rule::Tuple => Value::Array(parse_array(pair)),
        Rule::Object => Value::Object(parse_object(pair)),
        _ => Value::String(parse_expression(pair)),
    }
}

fn parse_array(pair: Pair<Rule>) -> Vec<Value> {
    pair.into_inner().map(parse_value).collect()
}

fn parse_object(pair: Pair<Rule>) -> Map<String, Value> {
    KeyValueIter::new(pair).collect()
}

fn parse_primitive<F>(pair: Pair<Rule>) -> F
where
    F: FromStr,
    <F as FromStr>::Err: std::fmt::Debug,
{
    pair.as_str().parse::<F>().unwrap()
}

fn inner(pair: Pair<Rule>) -> Pair<Rule> {
    pair.into_inner().next().unwrap()
}

fn parse_map_key(pair: Pair<Rule>) -> String {
    match pair.as_rule() {
        Rule::Identifier => parse_string(pair),
        Rule::StringLit => parse_string(inner(pair)),
        _ => parse_expression(pair),
    }
}

fn parse_string(pair: Pair<Rule>) -> String {
    pair.as_str().to_owned()
}

fn parse_expression(pair: Pair<Rule>) -> String {
    let expr = pair.as_str();
    let mut s = String::with_capacity(expr.len() + 3);
    s.push_str("${");
    s.push_str(expr);
    s.push('}');
    s
}

#[track_caller]
fn unexpected_rule(rule: Rule) -> ! {
    panic!("unexpected rule: {:?}", rule)
}

struct KeyValueIter<'a> {
    inner: Pairs<'a, Rule>,
}

impl<'a> KeyValueIter<'a> {
    fn new(pair: Pair<'a, Rule>) -> Self {
        KeyValueIter {
            inner: pair.into_inner(),
        }
    }
}

impl<'a> Iterator for KeyValueIter<'a> {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.inner.next(), self.inner.next()) {
            (Some(k), Some(v)) => Some((parse_map_key(k), parse_value(v))),
            (Some(k), None) => panic!("missing value for key: {}", k),
            (_, _) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pest::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn identifier() {
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
    fn string() {
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
    fn number() {
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
    fn attr() {
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
    fn collections() {
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
    fn template() {
        parses_to! {
            parser: HclParser,
            input: "<<HEREDOC\n${foo}\n%{if asdf}qux%{endif}\nheredoc\nHEREDOC",
            rule: Rule::ExprTerm,
            tokens: [
                Heredoc(0, 54, [
                    Identifier(2, 9),
                    Template(10, 46)
                ])
            ]
        };
    }

    #[test]
    fn cond_in_interpolation() {
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
    fn object_with_variable_expr_key() {
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
    fn nested_function_call_with_splat() {
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
    fn element_access_with_expression() {
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
    fn null_in_variable_expr() {
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
    fn escaped_slash_in_string() {
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
    fn test_parse_hcl() {
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
                                                "${aws_kms_key.mykey.arn}",
                                            ))
                                            .add_attribute(Attribute::new(
                                                "sse_algorithm",
                                                "aws:kms",
                                            ))
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
}
