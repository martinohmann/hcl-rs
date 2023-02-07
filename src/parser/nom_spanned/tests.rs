use super::ast::*;
use super::expr::expr;
use super::parse_to_end;
use super::repr::{Decor, Formatted, Spanned};
use crate::expr::{HeredocStripMode, Variable};
use crate::template::StripMode;
use crate::{Identifier, Number};
use indoc::indoc;
use pretty_assertions::assert_eq;

#[test]
fn parse_variable() {
    assert_eq!(
        parse_to_end("_an-id3nt1fieR", expr),
        Ok(Expression::Variable(Variable::unchecked("_an-id3nt1fieR")))
    );
}

#[test]
fn parse_string() {
    assert_eq!(
        parse_to_end("\"a string\"", expr),
        Ok(Expression::String("a string".into()))
    );

    assert_eq!(
        parse_to_end(r#""\\""#, expr),
        Ok(Expression::String("\\".into()))
    );
}

#[test]
fn parse_number() {
    assert_eq!(
        parse_to_end("12e+10", expr),
        Ok(Expression::Number(
            Number::from_f64(120000000000.0).unwrap()
        ))
    );

    assert_eq!(
        parse_to_end("42", expr),
        Ok(Expression::Number(Number::from(42u64)))
    );
}

#[test]
fn parse_conditional() {
    assert_eq!(
        parse_to_end("var.enabled ? 1 : 0", expr),
        Ok(Expression::Conditional(Box::new(Conditional {
            cond_expr: Formatted::new_with_span(
                Expression::Traversal(Box::new(Traversal {
                    expr: Formatted::new_with_span(
                        Expression::Variable(Variable::unchecked("var")),
                        0..3
                    ),
                    operators: vec![Formatted::new_with_span(
                        TraversalOperator::GetAttr(Identifier::unchecked("enabled")),
                        3..11,
                    )]
                })),
                0..11
            ),
            true_expr: Formatted::new_with_decor(
                Expression::Number(1.into()),
                14..15,
                Decor::from_prefix(13..14)
            ),
            false_expr: Formatted::new_with_decor(
                Expression::Number(0.into()),
                18..19,
                Decor::from_prefix(17..18)
            ),
        })))
    );
}

#[test]
fn parse_array() {
    assert_eq!(
        parse_to_end(r#"["bar", ["baz"]]"#, expr),
        Ok(Expression::Array(Box::new(Array::new(vec![
            Formatted::new_with_span(Expression::String("bar".into()), 1..6),
            Formatted::new_with_decor(
                Expression::Array(Box::new(Array::new(vec![Formatted::new_with_span(
                    Expression::String("baz".into()),
                    9..14
                )]))),
                8..15,
                Decor::from_prefix(7..8),
            ),
        ]))))
    );
}

#[test]
fn parse_object() {
    assert_eq!(
        parse_to_end(r#"{"bar" : "baz", "qux"= ident }"#, expr),
        Ok(Expression::Object(Box::new(Object::new(vec![
            {
                let mut item = ObjectItem::new(
                    Formatted::new_with_decor(
                        ObjectKey::Expression(Expression::String("bar".into())),
                        1..6,
                        Decor::from_suffix(6..7),
                    ),
                    Formatted::new_with_decor(
                        Expression::String("baz".into()),
                        9..14,
                        Decor::from_prefix(8..9),
                    ),
                );
                item.set_key_value_separator(ObjectKeyValueSeparator::Colon);
                item.set_value_terminator(ObjectValueTerminator::Comma);
                item
            },
            {
                let mut item = ObjectItem::new(
                    Formatted::new_with_decor(
                        ObjectKey::Expression(Expression::String("qux".into())),
                        16..21,
                        Decor::from_prefix(15..16),
                    ),
                    Formatted::new_with_decor(
                        Expression::Variable(Variable::unchecked("ident")),
                        23..28,
                        Decor::new(22..23, 28..29),
                    ),
                );
                item.set_value_terminator(ObjectValueTerminator::None);
                item
            },
        ]))))
    );

    assert_eq!(
        parse_to_end("{ #comment\n }", expr),
        Ok(Expression::Object(Box::new({
            let mut object = Object::new(vec![]);
            object.set_trailing(1..12);
            object
        })))
    );

    assert!(parse_to_end("{  }", expr).is_ok());
    assert!(parse_to_end("{ /*comment*/ }", expr).is_ok());
    assert!(parse_to_end("{ #comment\n }", expr).is_ok());
    assert!(parse_to_end("{ , }", expr).is_err());
    assert!(parse_to_end("{ foo = 1, }", expr).is_ok());
    assert!(parse_to_end("{ foo = 1 bar = 1 }", expr).is_err());
    assert!(parse_to_end("{ foo = 1, bar = 1 }", expr).is_ok());
    assert!(parse_to_end("{ foo = 1 /*comment*/ }", expr).is_ok());
    assert!(parse_to_end("{ foo = 1 #comment\n }", expr).is_ok());
    assert!(parse_to_end("{ foo = 1, #comment\n bar = 1 }", expr).is_ok());
}

#[test]
fn parse_heredoc() {
    assert_eq!(
        parse_to_end("<<HEREDOC\nHEREDOC", expr),
        Ok(Expression::HeredocTemplate(Box::new(HeredocTemplate {
            delimiter: Formatted::new_with_span(Identifier::unchecked("HEREDOC"), 2..9),
            template: Template {
                elements: Vec::new(),
                span: Some(10..10)
            },
            strip: HeredocStripMode::None,
        })))
    );

    assert_eq!(
        parse_to_end(
            indoc! {r#"
                <<HEREDOC
                ${foo}bar
                HEREDOC"#},
            expr,
        ),
        Ok(Expression::HeredocTemplate(Box::new(HeredocTemplate {
            delimiter: Formatted::new_with_span(Identifier::unchecked("HEREDOC"), 2..9),
            template: Template {
                elements: vec![
                    Element::Interpolation(Interpolation {
                        expr: Formatted::new_with_span(
                            Expression::Variable(Variable::unchecked("foo")),
                            2..5
                        ),
                        strip: StripMode::None,
                        span: Some(0..6),
                    }),
                    Element::Literal(Spanned {
                        value: "bar\n".into(),
                        span: Some(6..10)
                    }),
                ],
                span: Some(10..20)
            },
            strip: HeredocStripMode::None,
        })))
    );
}

// #[test]
// fn parse_template_expr() {
//     assert_eq!(
//         expr("\"foo ${bar} $${baz}, %{if cond ~} qux %{~ endif}\""),
//         Ok((
//             "",
//             Expression::from(TemplateExpr::from(
//                 "foo ${bar} $${baz}, %{ if cond ~} qux %{~ endif }"
//             ))
//         )),
//     );
// }

// #[test]
// fn parse_cond_in_interpolation() {
//     assert_eq!(
//         expr(r#""${var.l ? "us-east-1." : ""}""#),
//         Ok((
//             "",
//             Expression::from(TemplateExpr::from(r#"${var.l ? "us-east-1." : ""}"#))
//         )),
//     );
// }

// #[test]
// fn parse_nested_function_call_with_splat() {
//     assert_eq!(
//         expr("element(concat(aws_kms_key.key-one.*.arn, aws_kms_key.key-two.*.arn), 0)"),
//         Ok((
//             "",
//             Expression::from(
//                 FuncCall::builder("element")
//                     .arg(
//                         FuncCall::builder("concat")
//                             .arg(
//                                 Traversal::builder(Variable::unchecked("aws_kms_key"))
//                                     .attr("key-one")
//                                     .attr_splat()
//                                     .attr("arn")
//                                     .build()
//                             )
//                             .arg(
//                                 Traversal::builder(Variable::unchecked("aws_kms_key"))
//                                     .attr("key-two")
//                                     .attr_splat()
//                                     .attr("arn")
//                                     .build()
//                             )
//                             .build()
//                     )
//                     .arg(0)
//                     .build()
//             )
//         )),
//     );
// }

// #[test]
// fn parse_template() {
//     assert_eq!(
//         template("foo $${baz} ${bar}, %{if cond ~} qux %{~ endif}"),
//         Ok((
//             "",
//             Template::new()
//                 .add_literal("foo $${baz} ")
//                 .add_interpolation(Variable::unchecked("bar"))
//                 .add_literal(", ")
//                 .add_directive(
//                     IfDirective::new(
//                         Variable::unchecked("cond"),
//                         Template::new().add_literal(" qux ")
//                     )
//                     .with_if_strip(StripMode::End)
//                     .with_endif_strip(StripMode::Start)
//                 )
//         )),
//     );
// }

// #[test]
// fn parse_oneline_block() {
//     assert_eq!(
//         body("block { attr = 1 }"),
//         Ok((
//             "",
//             Body::builder()
//                 .add_block(Block::builder("block").add_attribute(("attr", 1)).build())
//                 .build()
//         ))
//     );
//     assert!(all_consuming(body)("block { attr = 1 attr2 = 2 }").is_err());
// }

// #[test]
// fn parse_body() {
//     assert_eq!(
//         body("foo = \"bar\"\nbar = 2"),
//         Ok((
//             "",
//             Body::builder()
//                 .add_attribute(("foo", "bar"))
//                 .add_attribute(("bar", 2u64))
//                 .build()
//         )),
//     );
// }
