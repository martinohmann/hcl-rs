mod common;

use common::{assert_format, assert_format_builder};
use hcl::expr::{
    BinaryOp, BinaryOperator, Conditional, Expression, ForExpr, FuncCall, Heredoc,
    HeredocStripMode, Traversal, TraversalOperator, Variable,
};
use hcl::format::Formatter;
use hcl::template::{ForDirective, IfDirective, Strip, Template};
use hcl::Identifier;
use indoc::indoc;

#[test]
fn prefer_ident_keys() {
    let attr = hcl::body!({
        object = {
            "foo" = 1
            bar = 2
            "baz qux" = 3
        }
    });

    assert_format_builder(
        Formatter::builder().prefer_ident_keys(false),
        &attr,
        indoc! {r#"
            object = {
              "foo" = 1
              bar = 2
              "baz qux" = 3
            }
        "#},
    );

    assert_format_builder(
        Formatter::builder().prefer_ident_keys(true),
        attr,
        indoc! {r#"
            object = {
              foo = 1
              bar = 2
              "baz qux" = 3
            }
        "#},
    );
}

#[test]
fn compact_arrays() {
    assert_format_builder(
        Formatter::builder().compact_arrays(true),
        hcl::body!({ array = [1, 2, 3, [4, 5]] }),
        indoc! {r#"
            array = [1, 2, 3, [4, 5]]
        "#},
    );
}

#[test]
fn compact_objects() {
    assert_format_builder(
        Formatter::builder().compact_objects(true),
        hcl::body!({
            object = {
                foo = {
                    bar = "baz"
                }
                qux = "bam"
            }
        }),
        indoc! {r#"
            object = { foo = { bar = "baz" }, qux = "bam" }
        "#},
    );
}

#[test]
fn compact_func_args() {
    assert_format(
        FuncCall::builder("func")
            .arg(vec![1, 2, 3])
            .arg(hcl::expression!({
                foo = "bar"
                baz = "qux"
            }))
            .build(),
        indoc! {r#"
            func([1, 2, 3], { foo = "bar", baz = "qux" })
        "#}
        .trim_end(),
    );
}

#[test]
fn func_call_expand_final() {
    assert_format(
        FuncCall::builder("func")
            .arg(1)
            .arg(vec!["two", "three"])
            .expand_final(true)
            .build(),
        indoc! {r#"
            func(1, ["two", "three"]...)
        "#}
        .trim_end(),
    );
}

#[test]
fn for_list_expr() {
    assert_format(
        ForExpr::new(
            Identifier::unchecked("item"),
            Variable::unchecked("items"),
            FuncCall::builder("func")
                .arg(Variable::unchecked("item"))
                .build(),
        )
        .with_cond_expr(Variable::unchecked("item")),
        "[for item in items : func(item) if item]",
    );
}

#[test]
fn for_object_expr() {
    assert_format(
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
        .with_cond_expr(BinaryOp::new(
            Variable::unchecked("value"),
            BinaryOperator::NotEq,
            Expression::Null,
        ))
        .with_grouping(true),
        "{for key, value in items : toupper(key) => tolower(value)... if value != null}",
    );
}

#[test]
fn conditional() {
    assert_format(
        Conditional::new(Variable::unchecked("cond_var"), "yes", "no"),
        r#"cond_var ? "yes" : "no""#,
    );
}

#[test]
fn operation() {
    assert_format(BinaryOp::new(1, BinaryOperator::Plus, 2), "1 + 2");
}

#[test]
fn template() {
    let template = Template::new().add_directive(
        ForDirective::new(
            Identifier::unchecked("item"),
            Variable::unchecked("items"),
            Template::new()
                .add_literal("\nHello ")
                .add_directive(
                    IfDirective::new(
                        BinaryOp::new(Variable::unchecked("item"), BinaryOperator::Eq, "world"),
                        Template::new().add_literal(" World! "),
                    )
                    .with_false_template(
                        Template::new()
                            .add_literal(" ")
                            .add_interpolation(Variable::unchecked("item"))
                            .add_literal("."),
                    )
                    .with_if_strip(Strip::Start)
                    .with_else_strip(Strip::Both)
                    .with_endif_strip(Strip::End),
                )
                .add_literal("\n"),
        )
        .with_for_strip(Strip::End)
        .with_endfor_strip(Strip::End),
    );

    let expected = indoc! {r#"
        %{ for item in items ~}
        Hello %{~ if item == "world" } World! %{~ else ~} ${item}.%{ endif ~}
        %{ endfor ~}
    "#}
    .trim_end();

    assert_format(template, expected);
}

#[test]
fn parenthesis() {
    assert_format(
        Expression::Parenthesis(Box::new(Variable::unchecked("foo").into())),
        "(foo)",
    );
}

#[test]
fn heredoc() {
    let body = hcl::body!({
        content {
            heredoc = (
                Heredoc::new(
                    Identifier::unchecked("HEREDOC"),
                    "foo\n  bar\nbaz\n"
                )
            )
        }
    });

    let expected = indoc! {r#"
        content {
          heredoc = <<HEREDOC
        foo
          bar
        baz
        HEREDOC
        }
    "#};

    assert_format(body, expected);
}

#[test]
fn indented_heredoc() {
    let body = hcl::body!({
        content {
            heredoc_indent = (
                Heredoc::new(
                    Identifier::unchecked("HEREDOC"),
                    "    foo\n      bar\n    baz\n",
                )
                .with_strip_mode(HeredocStripMode::Indent)
            )
        }
    });

    let expected = indoc! {r#"
        content {
          heredoc_indent = <<-HEREDOC
            foo
              bar
            baz
          HEREDOC
        }
    "#};

    assert_format(body, expected);
}

#[test]
fn traversal() {
    assert_format(
        Traversal::new(
            Variable::unchecked("var"),
            [
                TraversalOperator::GetAttr("foo".into()),
                TraversalOperator::FullSplat,
                TraversalOperator::GetAttr("bar".into()),
                TraversalOperator::Index(1u64.into()),
                TraversalOperator::AttrSplat,
                TraversalOperator::GetAttr("baz".into()),
                TraversalOperator::LegacyIndex(42),
            ],
        ),
        "var.foo[*].bar[1].*.baz.42",
    );
}

#[test]
fn empty_block() {
    assert_format(hcl::block!(empty {}), "empty {}\n");
}
