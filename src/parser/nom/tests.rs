use super::expr::expr;
use super::structure::body;
use super::template::template;
use crate::expr::{Conditional, Expression, FuncCall, Heredoc, TemplateExpr, Traversal, Variable};
use crate::structure::{Block, Body};
use crate::template::{IfDirective, StripMode, Template};
use crate::{Identifier, Number};
use indexmap::indexmap;
use indoc::indoc;
use nom::combinator::all_consuming;
use pretty_assertions::assert_eq;

#[test]
fn parse_variable() {
    assert_eq!(
        expr("_an-id3nt1fieR"),
        Ok(("", Expression::from(Variable::unchecked("_an-id3nt1fieR"))))
    );
}

#[test]
fn parse_string() {
    assert_eq!(expr("\"a string\""), Ok(("", Expression::from("a string"))));

    assert_eq!(expr(r#""\\""#), Ok(("", Expression::from("\\"))));
}

#[test]
fn parse_number() {
    assert_eq!(
        expr("12e+10"),
        Ok((
            "",
            Expression::from(Number::from_f64(120000000000.0).unwrap())
        ))
    );

    assert_eq!(expr("42"), Ok(("", Expression::from(Number::from(42u64)))));
}

#[test]
fn parse_conditional() {
    assert_eq!(
        expr("var.enabled ? 1 : 0"),
        Ok((
            "",
            Expression::from(Conditional::new(
                Traversal::builder(Variable::unchecked("var"))
                    .attr("enabled")
                    .build(),
                1,
                0
            ))
        ))
    );
}

#[test]
fn parse_array() {
    assert_eq!(
        expr(r#"["bar", ["baz"]]"#),
        Ok((
            "",
            Expression::from(vec![Expression::from("bar"), Expression::from(vec!["baz"])])
        ))
    );
}

#[test]
fn parse_object() {
    assert_eq!(
        expr(r#"{"bar" = "baz","qux" = ident }"#),
        Ok((
            "",
            Expression::from_iter(indexmap! {
                "bar" => Expression::from("baz"),
                "qux" => Expression::from(Variable::unchecked("ident")),
            }),
        ))
    );
}

#[test]
fn parse_heredoc() {
    assert_eq!(
        expr("<<HEREDOC\nHEREDOC\n"),
        Ok((
            "\n",
            Expression::from(Heredoc::new(Identifier::unchecked("HEREDOC"), "")),
        ))
    );

    assert_eq!(
        expr(indoc! {r#"
            <<HEREDOC
            ${foo}
            %{if asdf}qux%{endif}
            heredoc
            HEREDOC
        "#}),
        Ok((
            "\n",
            Expression::from(Heredoc::new(
                Identifier::unchecked("HEREDOC"),
                "${foo}\n%{ if asdf }qux%{ endif }\nheredoc\n"
            )),
        ))
    );
}

#[test]
fn parse_template_expr() {
    assert_eq!(
        expr("\"foo ${bar} $${baz}, %{if cond ~} qux %{~ endif}\""),
        Ok((
            "",
            Expression::from(TemplateExpr::from(
                "foo ${bar} $${baz}, %{ if cond ~} qux %{~ endif }"
            ))
        )),
    );
}

#[test]
fn parse_cond_in_interpolation() {
    assert_eq!(
        expr(r#""${var.l ? "us-east-1." : ""}""#),
        Ok((
            "",
            Expression::from(TemplateExpr::from(r#"${var.l ? "us-east-1." : ""}"#))
        )),
    );
}

#[test]
fn parse_nested_function_call_with_splat() {
    assert_eq!(
        expr("element(concat(aws_kms_key.key-one.*.arn, aws_kms_key.key-two.*.arn), 0)"),
        Ok((
            "",
            Expression::from(
                FuncCall::builder("element")
                    .arg(
                        FuncCall::builder("concat")
                            .arg(
                                Traversal::builder(Variable::unchecked("aws_kms_key"))
                                    .attr("key-one")
                                    .attr_splat()
                                    .attr("arn")
                                    .build()
                            )
                            .arg(
                                Traversal::builder(Variable::unchecked("aws_kms_key"))
                                    .attr("key-two")
                                    .attr_splat()
                                    .attr("arn")
                                    .build()
                            )
                            .build()
                    )
                    .arg(0)
                    .build()
            )
        )),
    );
}

#[test]
fn parse_template() {
    assert_eq!(
        template("foo $${baz} ${bar}, %{if cond ~} qux %{~ endif}"),
        Ok((
            "",
            Template::new()
                .add_literal("foo $${baz} ")
                .add_interpolation(Variable::unchecked("bar"))
                .add_literal(", ")
                .add_directive(
                    IfDirective::new(
                        Variable::unchecked("cond"),
                        Template::new().add_literal(" qux ")
                    )
                    .with_if_strip(StripMode::End)
                    .with_endif_strip(StripMode::Start)
                )
        )),
    );
}

#[test]
fn parse_oneline_block() {
    assert_eq!(
        body("block { attr = 1 }"),
        Ok((
            "",
            Body::builder()
                .add_block(Block::builder("block").add_attribute(("attr", 1)).build())
                .build()
        ))
    );
    assert!(all_consuming(body)("block { attr = 1 attr2 = 2 }").is_err());
}

#[test]
fn parse_body() {
    assert_eq!(
        body("foo = \"bar\"\nbar = 2"),
        Ok((
            "",
            Body::builder()
                .add_attribute(("foo", "bar"))
                .add_attribute(("bar", 2u64))
                .build()
        )),
    );
}
