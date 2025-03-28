use super::expr::expr;
use super::parse_complete;
use super::structure::body;
use super::template::template;
use crate::{expr::Expression, Formatted, Number};
use indoc::indoc;
use pretty_assertions::assert_eq;

macro_rules! assert_roundtrip {
    ($input:expr, $parser:expr) => {
        let mut parsed = parse_complete($input, $parser).unwrap();
        parsed.despan($input);
        assert_eq!(&parsed.to_string(), $input);
    };
}

#[test]
fn number_expr() {
    let parsed = parse_complete("42", expr).unwrap();
    let expected = Expression::Number(Formatted::new(Number::from(42)));
    assert_eq!(parsed, expected);
}

#[test]
fn roundtrip_expr() {
    let inputs = [
        "_an-id3nt1fieR",
        r#""a string""#,
        r#""\\""#,
        "12e+10",
        "- 12e+10",
        "-34.0012e+10",
        "-1.0000",
        "1.0000E10",
        "42",
        "var.enabled ? 1 : 0",
        r#"["bar", ["baz"]]"#,
        "[1,]",
        r#"[format("prefix-%s", var.foo)]"#,
        r#"{"bar" = "baz","qux" = ident }"#,
        "{\"bar\" : \"baz\", \"qux\"= ident # a comment\n }",
        "{ #comment\n }",
        "{  }",
        "{ /*comment*/ }",
        "{ foo = 1, }",
        "{ foo = 1, bar = 1 }",
        "{ foo = 1 /*comment*/ }",
        "{ foo = 1 #comment\n }",
        "{ foo = 1, #comment\n bar = 1 }",
        "<<HEREDOC\nHEREDOC",
        indoc! {r"
            <<HEREDOC
            ${foo}
            %{if asdf}qux%{endif}
            heredoc
            HEREDOC"},
        r#""foo ${bar} $${baz}, %{if cond ~} qux %{~ endif}""#,
        r#""${var.l ? "us-east-1." : ""}""#,
        "element(concat(aws_kms_key.key-one.*.arn, aws_kms_key.key-two.*.arn), 0)",
        "foo::bar(baz...)",
        "foo :: bar ()",
        "foo(bar...)",
        "foo(bar,)",
        "foo( )",
    ];

    for input in inputs {
        assert_roundtrip!(input, expr);
    }
}

#[test]
fn roundtrip_body() {
    let mut inputs = vec![
        indoc! {r#"
            // comment
            block {
              foo = "bar"
            }

            oneline { bar="baz"} # comment

            array = [
              1, 2,
              3
            ]
        "#},
        "block { attr = 1 }\n",
        "foo = \"bar\"\nbar = 2\n",
        "foo = \"bar\"\nbar = 3",
        indoc! {r"
            indented_heredoc = <<-EOT
                ${foo}
              %{if asdf}qux%{endif}bar
                  heredoc
                EOT
        "},
        "ami = \"ami-5f6495430e7781fe5\" // Ubuntu 20.04 LTS\n",
        "ami = \"ami-5f6495430e7781fe5\" /* Ubuntu 20.04 LTS */\n",
        "array =   [1, 2, 3]\n",
        "block {}\n\n// trailing body comment",
    ];

    let tests = testdata::load().unwrap();
    assert!(!tests.is_empty());

    for test in &tests {
        inputs.push(&test.input);
    }

    for input in inputs {
        assert_roundtrip!(input, body);
    }
}

#[test]
fn roundtrip_template() {
    let inputs = [
        "foo $${baz} ${bar}, %{if cond ~} qux %{~ endif}",
        indoc! {r"
            Bill of materials:
            %{ for item in items ~}
            - ${item}
            %{ endfor ~}
        "},
        "literal $${escaped} ${value}",
    ];

    for input in inputs {
        assert_roundtrip!(input, template);
    }
}

#[test]
fn invalid_exprs() {
    let inputs = [
        "{ , }",
        "[ , ]",
        "{ foo = 1 bar = 1 }",
        "foo(...)",
        "foo(,)",
    ];

    for input in inputs {
        assert!(
            parse_complete(input, expr).is_err(),
            "expected expression to be invalid: `{input}`",
        );
    }
}
