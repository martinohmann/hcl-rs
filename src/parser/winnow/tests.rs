use super::ast::*;
use super::expr::expr;
use super::parse_to_end;
use super::repr::{Decor, Decorate, Decorated, Despan, Span, Spanned};
use super::structure::body;
use crate::expr::Variable;
use crate::template::StripMode;
use crate::{Identifier, InternalString, Number};
use indoc::indoc;
use pretty_assertions::assert_eq;

#[test]
fn parse_variable() {
    assert_eq!(
        parse_to_end("_an-id3nt1fieR", expr),
        Ok(Expression::Variable(Decorated::new(Variable::unchecked(
            "_an-id3nt1fieR"
        ))))
    );
}

#[test]
fn parse_string() {
    assert_eq!(
        parse_to_end("\"a string\"", expr),
        Ok(Expression::String(Decorated::new("a string".into())))
    );

    assert_eq!(
        parse_to_end(r#""\\""#, expr),
        Ok(Expression::String(Decorated::new("\\".into())))
    );
}

#[test]
fn parse_number() {
    assert_eq!(
        parse_to_end("12e+10", expr),
        Ok(Expression::Number(Decorated::new(
            Number::from_f64(120000000000.0).unwrap()
        )))
    );

    assert_eq!(
        parse_to_end("42", expr),
        Ok(Expression::Number(Decorated::new(Number::from(42u64))))
    );
}

#[test]
fn parse_conditional() {
    assert_eq!(
        parse_to_end("var.enabled ? 1 : 0", expr),
        Ok(Expression::Conditional(Box::new(Conditional::new(
            Expression::Traversal(Box::new(
                Traversal::new(
                    Expression::Variable(Decorated::new(Variable::unchecked("var")).spanned(0..3)),
                    vec![Decorated::new(TraversalOperator::GetAttr(
                        Decorated::new(Identifier::unchecked("enabled"))
                            .spanned(4..11)
                            .decorated(Decor::from_prefix(""))
                    ))
                    .spanned(3..11)
                    .decorated(Decor::from_prefix(""))]
                )
                .spanned(0..11)
                .decorated(Decor::from_suffix(11..12)),
            )),
            Expression::Number(
                Decorated::new(Number::from(1))
                    .spanned(14..15)
                    .decorated(Decor::new(13..14, 15..16))
            ),
            Expression::Number(
                Decorated::new(Number::from(0))
                    .spanned(18..19)
                    .decorated(Decor::from_prefix(17..18))
            ),
        ))))
    );
}

#[test]
fn parse_array() {
    assert_eq!(
        parse_to_end(r#"["bar", ["baz"]]"#, expr),
        Ok(Expression::Array(Box::new(Array::new(vec![
            Expression::String(
                Decorated::new(InternalString::from("bar"))
                    .spanned(1..6)
                    .decorated(Decor::new("", ""))
            ),
            Expression::Array(Box::new(
                Array::new(vec![Expression::String(
                    Decorated::new(InternalString::from("baz"))
                        .spanned(9..14)
                        .decorated(("", ""))
                )])
                .spanned(8..15)
                .decorated((7..8, ""))
            ))
        ]))))
    );
}

#[test]
fn parse_object() {
    assert_eq!(
        parse_to_end("{\"bar\" : \"baz\", \"qux\"= ident # a comment\n }", expr),
        Ok(Expression::Object(Box::new({
            let mut object = Object::new(vec![
                {
                    let mut item = ObjectItem::new(
                        ObjectKey::Expression(Expression::String(
                            Decorated::new(InternalString::from("bar"))
                                .spanned(1..6)
                                .decorated(Decor::new("", 6..7)),
                        )),
                        Expression::String(
                            Decorated::new(InternalString::from("baz"))
                                .spanned(9..14)
                                .decorated(Decor::new(8..9, "")),
                        ),
                    )
                    .spanned(1..15);
                    item.set_key_value_separator(ObjectKeyValueSeparator::Colon);
                    item.set_value_terminator(ObjectValueTerminator::Comma);
                    item
                },
                {
                    let mut item = ObjectItem::new(
                        ObjectKey::Expression(Expression::String(
                            Decorated::new(InternalString::from("qux"))
                                .spanned(16..21)
                                .decorated(Decor::new(15..16, "")),
                        )),
                        Expression::Variable(
                            Decorated::new(Variable::unchecked("ident"))
                                .spanned(23..28)
                                .decorated(Decor::new(22..23, 28..40)),
                        ),
                    )
                    .spanned(15..41);
                    item.set_value_terminator(ObjectValueTerminator::Newline);
                    item
                },
            ]);
            object.set_trailing(41..42);
            object.into()
        })))
    );

    assert_eq!(
        parse_to_end("{ #comment\n }", expr),
        Ok(Expression::Object(Box::new({
            let mut object = Object::new(vec![]);
            object.set_trailing(1..12);
            object.into()
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
#[ignore]
fn parse_heredoc() {
    assert_eq!(
        parse_to_end("<<HEREDOC\nHEREDOC", expr),
        Ok(Expression::HeredocTemplate(Box::new(HeredocTemplate::new(
            Decorated::new(Identifier::unchecked("HEREDOC")).spanned(2..9),
            Template::default().spanned(10..10),
        ))))
    );

    assert_eq!(
        parse_to_end(
            indoc! {r#"
                <<HEREDOC
                ${foo}bar
                HEREDOC"#},
            expr,
        ),
        Ok(Expression::HeredocTemplate(Box::new(HeredocTemplate::new(
            Decorated::new(Identifier::unchecked("HEREDOC")).spanned(2..9),
            Template::new(vec![
                Element::Interpolation(
                    Interpolation::new(
                        Expression::Variable(
                            Decorated::new(Variable::unchecked("foo"),)
                                .spanned(12..15)
                                .decorated(("", ""))
                        ),
                        StripMode::None
                    )
                    .spanned(10..16),
                ),
                Element::Literal(Spanned::new(InternalString::from("bar\n")).spanned(16..20)),
            ])
            .spanned(10..20),
        ))))
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

macro_rules! assert_roundtrip {
    ($input:expr, $parser:expr) => {
        let mut parsed = parse_to_end($input, $parser).unwrap();
        parsed.despan($input);
        assert_eq!(&parsed.to_string(), $input);
    };
}

#[test]
fn roundtrip_body() {
    let input = indoc! {r#"
        // comment
        block {
          foo = "bar"
        }

        oneline { bar="baz"} # comment

        array = [
          1, 2,
          3
        ]
    "#};

    assert_roundtrip!(input, body);
}

#[test]
fn roundtrip_large() {
    let input = std::fs::read_to_string("benches/network.tf").unwrap();

    assert_roundtrip!(&input, body);
}
