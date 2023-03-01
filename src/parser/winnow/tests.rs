use super::ast::*;
use super::expr::expr;
use super::parse_to_end;
use super::repr::{Decorated, Despan, SetSpan};
use super::structure::body;
use super::template::template;
use crate::Number;
use indoc::indoc;
use pretty_assertions::assert_eq;
use std::fs;
use std::io;

fn load_tests() -> Result<Vec<String>, io::Error> {
    let mut tests = Vec::new();

    for entry in fs::read_dir("testdata")? {
        let path = entry?.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "hcl") {
            let content = fs::read_to_string(&path)?;
            tests.push(content);
        }
    }

    tests.sort();

    Ok(tests)
}

#[test]
fn parse_number() {
    assert_eq!(
        parse_to_end("12e+10", expr),
        Ok(Expression::Number(
            Decorated::new(Number::from_f64(120000000000.0).unwrap()).spanned(0..6)
        ))
    );
}

macro_rules! assert_roundtrip {
    ($input:expr, $parser:expr) => {
        let mut parsed = parse_to_end($input, $parser).unwrap();
        parsed.despan($input);
        assert_eq!(&parsed.to_string(), $input);
    };
}

#[test]
fn roundtrip_expr() {
    let inputs = [
        "_an-id3nt1fieR",
        r#""a string""#,
        r#""\\""#,
        // "12e+10",
        "42",
        "var.enabled ? 1 : 0",
        r#"["bar", ["baz"]]"#,
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
        indoc! {r#"
            <<HEREDOC
            ${foo}
            %{if asdf}qux%{endif}
            heredoc
            HEREDOC"#},
        r#""foo ${bar} $${baz}, %{if cond ~} qux %{~ endif}""#,
        r#""${var.l ? "us-east-1." : ""}""#,
        "element(concat(aws_kms_key.key-one.*.arn, aws_kms_key.key-two.*.arn), 0)",
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
    ];

    let tests = load_tests().unwrap();
    assert!(tests.len() > 0);

    for input in &tests {
        inputs.push(input);
    }

    for input in inputs {
        assert_roundtrip!(input, body);
    }
}

#[test]
fn roundtrip_template() {
    let inputs = [
        "foo $${baz} ${bar}, %{if cond ~} qux %{~ endif}",
        indoc! {r#"
            Bill of materials:
            %{ for item in items ~}
            - ${item}
            %{ endfor ~}
        "#},
    ];

    for input in inputs {
        assert_roundtrip!(input, template);
    }
}

#[test]
fn invalid_exprs() {
    let inputs = ["{ , }", "{ foo = 1 bar = 1 }"];

    for input in inputs {
        assert!(parse_to_end(input, expr).is_err());
    }
}
