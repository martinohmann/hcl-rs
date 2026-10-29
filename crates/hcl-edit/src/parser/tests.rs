use super::expr::expr;
use super::parse_complete;
use super::structure::body;
use super::template::template;
use crate::{
    expr::{BinaryOp, BinaryOperator, Expression, Parenthesis},
    Formatted, Number,
};
use indoc::indoc;
use pretty_assertions::assert_eq;
use testdata::nesting::{binary_inputs, nested_inputs, with_small_stack};

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
fn neg_number_expr() {
    let tests = [
        ("-9223372036854775808", Number::from(i64::MIN)),
        (
            "-9223372036854775809",
            Number::from_f64(-9_223_372_036_854_775_809.0).unwrap(),
        ),
        (
            "-18446744073709551615",
            Number::from_f64(-18_446_744_073_709_551_615.0).unwrap(),
        ),
    ];

    for (input, expected) in tests {
        let parsed = parse_complete(input, expr).unwrap();
        let Expression::Number(num) = parsed else {
            panic!("expected number expression for `{input}`, got {parsed:?}");
        };
        assert_eq!(*num.value(), expected, "unexpected value for `{input}`");
        assert!(
            num.value().as_f64().unwrap() < 0.0,
            "`{input}` must be negative"
        );
    }
}

#[test]
fn binary_ops() {
    use BinaryOperator::*;

    let tests = [
        (
            "1 + 1 == 2",
            BinaryOp::new(BinaryOp::new(1, Plus, 1), Eq, 2),
        ),
        (
            "1 + 1 * 2 / 3",
            BinaryOp::new(1, Plus, BinaryOp::new(BinaryOp::new(1, Mul, 2), Div, 3)),
        ),
        (
            "(1 + 1) * 2 / 3",
            BinaryOp::new(
                BinaryOp::new(Parenthesis::new(BinaryOp::new(1, Plus, 1).into()), Mul, 2),
                Div,
                3,
            ),
        ),
        (
            "(1 + 1) * (2 / 3)",
            BinaryOp::new(
                Parenthesis::new(BinaryOp::new(1, Plus, 1).into()),
                Mul,
                Parenthesis::new(BinaryOp::new(2, Div, 3).into()),
            ),
        ),
        // Comparison operators are left-associative like all other binary operators.
        (
            "1 == 1 == true",
            BinaryOp::new(BinaryOp::new(1, Eq, 1), Eq, true),
        ),
        (
            "1 != 2 == true",
            BinaryOp::new(BinaryOp::new(1, NotEq, 2), Eq, true),
        ),
        (
            "1 < 2 < 3",
            BinaryOp::new(BinaryOp::new(1, Less, 2), Less, 3),
        ),
        (
            "1 <= 2 >= 3 > 4",
            BinaryOp::new(
                BinaryOp::new(BinaryOp::new(1, LessEq, 2), GreaterEq, 3),
                Greater,
                4,
            ),
        ),
        (
            "1 < 2 == 3 > 4",
            BinaryOp::new(BinaryOp::new(1, Less, 2), Eq, BinaryOp::new(3, Greater, 4)),
        ),
    ];

    for (given, expected) in tests {
        let parsed = parse_complete(given, expr).unwrap();
        assert_eq!(Expression::from(expected), parsed);
    }
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
        "1 == 1 == true",
        "1 < 2 < 3 != false",
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
        "a = 1 + 2 - 3 * 4 / 5 // comment\n",
        "c = 1 + 2 - 3 * 4 / 5 /* block comment */\n",
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
fn heredoc_prefix_delimiter() {
    // Empty heredoc (non-indented) followed by heredoc with prefix-sharing delimiter.
    let input = "a = <<EOF\nEOF\nb = <<EOFTF\ncontent\nEOFTF\n";
    assert_roundtrip!(input, body);

    // Non-empty indented heredoc followed by heredoc with prefix-sharing delimiter.
    let input = indoc! {r#"
        a = <<-EOF
          hello
          EOF
        b = <<-EOFTF
          world
          EOFTF
    "#};
    assert_roundtrip!(input, body);

    // Non-empty non-indented heredoc followed by heredoc with prefix-sharing delimiter.
    let input = "a = <<EOF\nhello\nEOF\nb = <<EOFTF\nworld\nEOFTF\n";
    assert_roundtrip!(input, body);

    // Empty indented heredoc followed by prefix-sharing delimiter parses successfully.
    let input = "a = <<-EOF\nEOF\nb = <<-EOFTF\ncontent\nEOFTF\n";
    parse_complete(input, body).expect("should parse empty <<-EOF followed by <<-EOFTF");
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

#[test]
fn valid_escape_sequences() {
    let tests = [
        (r#""a\nb""#, "a\nb"),
        (r#""a\rb""#, "a\rb"),
        (r#""a\tb""#, "a\tb"),
        (r#""a\\b""#, "a\\b"),
        (r#""a\"b""#, "a\"b"),
        (r#""a\u2665b""#, "a\u{2665}b"),
        (r#""a\U0001f600b""#, "a\u{1f600}b"),
    ];

    for (input, expected) in tests {
        let parsed = parse_complete(input, expr).unwrap();
        assert_eq!(parsed.as_str(), Some(expected), "input: `{input}`");
    }
}

#[test]
fn invalid_escape_sequences() {
    // An invalid escape selector must raise a parse error rather than silently
    // dropping the escaped character. The leading-literal cases regress the
    // variant where the invalid escape reaches the string-building loop.
    let inputs = [
        r#""\q""#,
        r#""pre\qpost""#,
        r#""z\ay""#,
        r#""\x41""#,
        r#""\0""#,
        r#""\v""#,
        r#""\'""#,
        r#""\e""#,
        // JSON-only escapes which are not part of the HCL spec.
        r#""\b""#,
        r#""\f""#,
        r#""\/""#,
        r#""pre\bpost""#,
    ];

    for input in inputs {
        assert!(
            parse_complete(input, expr).is_err(),
            "expected invalid escape sequence to be rejected: `{input}`",
        );
    }
}

fn assert_nesting_error(name: &str, err: &crate::parser::Error) {
    assert_eq!(
        err.message(),
        "invalid nesting depth; expected at most 64 nested levels",
        "unexpected error for {name}"
    );
}

fn assert_operator_depth_error(name: &str, err: &crate::parser::Error) {
    assert_eq!(
        err.message(),
        "invalid binary operation; expected at most 128 chained binary operators",
        "unexpected error for {name}"
    );
}

#[test]
fn max_nesting_depth() {
    with_small_stack(|| {
        for (name, _) in nested_inputs(0) {
            // Find the deepest nesting that is accepted and make sure that handling it does not
            // overflow the stack.
            let mut depth = 0;

            loop {
                let (_, input) = nested_inputs(depth + 1)
                    .into_iter()
                    .find(|(n, _)| *n == name)
                    .unwrap();

                match crate::parser::parse_body(&input) {
                    Ok(body) => {
                        assert_eq!(body.to_string(), input, "roundtrip failed for {name}");
                        drop(body.clone());
                        depth += 1;
                    }
                    Err(err) => {
                        assert_nesting_error(name, &err);
                        break;
                    }
                }
            }

            assert!(
                depth >= 60,
                "expected at least 60 nesting levels for {name}, got {depth}"
            );
        }
    });
}

#[test]
fn max_operator_depth() {
    with_small_stack(|| {
        for (name, _) in binary_inputs(0) {
            // Find the deepest operator chain that is accepted and make sure that handling it
            // does not overflow the stack.
            let mut depth = 0;

            loop {
                let (_, input) = binary_inputs(depth + 1)
                    .into_iter()
                    .find(|(n, _)| *n == name)
                    .unwrap();

                match crate::parser::parse_body(&input) {
                    Ok(body) => {
                        assert_eq!(body.to_string(), input, "roundtrip failed for {name}");
                        drop(body.clone());
                        depth += 1;
                    }
                    Err(err) => {
                        assert_operator_depth_error(name, &err);
                        break;
                    }
                }
            }

            assert_eq!(depth, 128, "unexpected max operator depth for {name}");
        }
    });
}

#[test]
fn deeply_nested_input_is_rejected() {
    with_small_stack(|| {
        for (name, input) in nested_inputs(100_000) {
            let err = crate::parser::parse_body(&input).unwrap_err();
            assert_nesting_error(name, &err);
        }

        for (name, input) in binary_inputs(100_000) {
            let err = crate::parser::parse_body(&input).unwrap_err();
            assert_operator_depth_error(name, &err);
        }
    });
}

#[test]
fn nesting_depth_is_restored_after_nested_parsers() {
    // Many sibling expressions each nested close to the limit must not accumulate depth.
    let item = format!("{}1{}", "[".repeat(50), "]".repeat(50));
    let input = format!(
        "x = [{}]\ny = 1{}\na {{\n  b = {}\n}}\n",
        vec![item.as_str(); 200].join(", "),
        " + 1".repeat(50),
        item
    );
    assert!(crate::parser::parse_body(&input).is_ok());
}

#[test]
fn operator_depth_of_siblings_does_not_add_up() {
    // Sibling expressions with long operator chains each stay within the limit, so they must be
    // accepted. Only operators on the same path through the expression tree add up.
    let chain = format!("1{}", " + 1".repeat(100));
    let input = format!(
        "a = [{chain}, {chain}]\n\
         b = {{ x = {chain}, y = {chain} }}\n\
         c = f({chain}, {chain})\n\
         d = {chain} == 1 ? {chain} : {chain}\n\
         e = \"${{{chain}}}${{{chain}}}\"\n\
         f = a[{chain}][{chain}]\n"
    );
    assert!(crate::parser::parse_body(&input).is_ok());
}

#[test]
fn templates_without_nested_elements_at_max_nesting_depth() {
    // Templates which only contain literals do not take a nesting level, so they must be accepted
    // at the maximum nesting depth. Every template ends with an attempt to parse a directive,
    // which must not count as nesting.
    with_small_stack(|| {
        let templates = [
            "\"foo\"",
            "\"foo\\nbar\"",
            "<<EOT\nfoo\nEOT\n",
            "<<-EOT\n  foo\n  EOT\n",
            "<<EOT\nEOT\n",
        ];

        for template in templates {
            let input = format!("x = {}{template}{}", "[".repeat(64), "]".repeat(64));
            let body = crate::parser::parse_body(&input).unwrap_or_else(|err| {
                panic!("expected `{template}` to be accepted at max nesting depth: {err}")
            });
            assert_eq!(body.to_string(), input);
        }
    });
}
