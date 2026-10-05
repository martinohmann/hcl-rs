use hcl_edit::parser::parse_body;
use indoc::indoc;
use pretty_assertions::assert_eq;

macro_rules! assert_error {
    ($hcl:expr, $msg:expr) => {
        match parse_body($hcl) {
            Ok(s) => panic!("parsed to: {:#?}", s),
            Err(e) => assert_eq!($msg, e.to_string()),
        }
    };
}

#[test]
fn invalid_structures() {
    assert_error!(
        "foo = 1\nbar [",
        indoc! {r#"
             --> HCL parse error in line 2, column 5
              |
            2 | bar [
              |     ^---
              |
              = invalid structure; expected `{`, `=`, `"` or identifier"#}
    );

    assert_error!(
        "foo = 1\nfoo = 2",
        indoc! {r#"
             --> HCL parse error in line 2, column 1
              |
            2 | foo = 2
              | ^---
              |
              = invalid attribute; expected unique attribute key; found redefined attribute"#}
    );
}

#[test]
fn invalid_blocks() {
    assert_error!(
        "ident {",
        indoc! {r#"
             --> HCL parse error in line 1, column 8
              |
            1 | ident {
              |        ^---
              |
              = invalid block body; expected `}`, newline or identifier"#}
    );

    assert_error!(
        "ident \"label\" {",
        indoc! {r#"
             --> HCL parse error in line 1, column 16
              |
            1 | ident "label" {
              |                ^---
              |
              = invalid block body; expected `}`, newline or identifier"#}
    );

    assert_error!(
        "ident { foo }",
        indoc! {r#"
             --> HCL parse error in line 1, column 13
              |
            1 | ident { foo }
              |             ^---
              |
              = invalid attribute; expected `=`"#}
    );

    assert_error!(
        "ident { [ }",
        indoc! {r#"
             --> HCL parse error in line 1, column 9
              |
            1 | ident { [ }
              |         ^---
              |
              = invalid block body; expected `}`, newline or identifier"#}
    );
}

#[test]
fn invalid_exprs() {
    assert_error!(
        "ident = ''",
        indoc! {r#"
             --> HCL parse error in line 1, column 9
              |
            1 | ident = ''
              |         ^---
              |
              = invalid expression; expected `"`, `[`, `{`, `-`, `!`, `(`, `_`, `<`, letter or digit"#}
    );

    assert_error!(
        "ident = var.%",
        indoc! {r#"
             --> HCL parse error in line 1, column 13
              |
            1 | ident = var.%
              |             ^---
              |
              = invalid traversal operator; expected `*`, identifier or unsigned integer"#}
    );

    assert_error!(
        "ident = { foo = \"\"\" }",
        indoc! {r#"
             --> HCL parse error in line 1, column 19
              |
            1 | ident = { foo = """ }
              |                   ^---
              |
              = invalid object item; expected `}`, `,` or newline"#}
    );

    assert_error!(
        "ident = foo::",
        indoc! {r#"
             --> HCL parse error in line 1, column 14
              |
            1 | ident = foo::
              |              ^---
              |
              = expected identifier"#}
    );

    assert_error!(
        "ident = foo::bar",
        indoc! {r#"
             --> HCL parse error in line 1, column 17
              |
            1 | ident = foo::bar
              |                 ^---
              |
              = expected `(`"#}
    );

    assert_error!(
        "ident = foo( ",
        indoc! {r#"
             --> HCL parse error in line 1, column 14
              |
            1 | ident = foo( 
              |              ^---
              |
              = expected `)` or expression"#}
    );

    assert_error!(
        "unicodé_ident = '4",
        indoc! {r#"
             --> HCL parse error in line 1, column 17
              |
            1 | unicodé_ident = '4
              |                 ^---
              |
              = invalid expression; expected `"`, `[`, `{`, `-`, `!`, `(`, `_`, `<`, letter or digit"#}
    );
}

#[test]
fn redefined_object_keys() {
    assert_error!(
        "ident = { foo = 1, foo = 2 }",
        indoc! {r#"
             --> HCL parse error in line 1, column 20
              |
            1 | ident = { foo = 1, foo = 2 }
              |                    ^---
              |
              = invalid object item; expected unique object key; found redefined object key"#}
    );

    assert_error!(
        "ident = { \"foo\" = 1, \"foo\" = 2 }",
        indoc! {r#"
             --> HCL parse error in line 1, column 22
              |
            1 | ident = { "foo" = 1, "foo" = 2 }
              |                      ^---
              |
              = invalid object item; expected unique object key; found redefined object key"#}
    );

    assert_error!(
        indoc! {r#"
            ident = {
              foo = 1
              foo = 2
            }"#},
        indoc! {r#"
             --> HCL parse error in line 3, column 3
              |
            3 |   foo = 2
              |   ^---
              |
              = invalid object item; expected unique object key; found redefined object key"#}
    );
}

// Builds the expected error with `message` at `column` of `line`.
fn expected_error(line: usize, column: usize, line_content: &str, message: &str) -> String {
    let s = " ".repeat(line.to_string().len());
    let caret = " ".repeat(column - 1);
    format!(
        "{s}--> HCL parse error in line {line}, column {column}\n\
         {s} |\n\
         {line} | {line_content}\n\
         {s} | {caret}^---\n\
         {s} |\n\
         {s} = {message}"
    )
}

#[test]
fn max_nesting_depth_exceeded() {
    fn expected(line: usize, column: usize, line_content: &str) -> String {
        let message = "invalid nesting depth; expected at most 64 nested levels";
        expected_error(line, column, line_content, message)
    }

    // The attribute value itself does not count towards the nesting depth, but each array element
    // takes one level. After 65 `[` the limit is reached, so the element starting at the 66th `[`
    // is one level too deep. `x = ` takes 4 columns.
    let input = format!("x = {}1{}", "[".repeat(200), "]".repeat(200));
    assert_error!(&input, expected(1, 4 + 65 + 1, &input));

    // Same with leading content on the line and a preceding line. `foo = [1, {a = ` takes 15
    // columns and 2 levels (array element, object value), so the limit is reached after 63 `[`
    // and the error is reported at the 64th.
    let line = format!("foo = [1, {{a = {}1{}}}]", "[".repeat(200), "]".repeat(200));
    let input = format!("bar = 1\n{line}");
    assert_error!(&input, expected(2, 15 + 63 + 1, &line));

    // Block bodies take one level, so within a block the limit is reached after 64 `[` and the
    // error is reported at the 65th. `  x = ` takes 6 columns.
    let line = format!("  x = {}1{}", "[".repeat(200), "]".repeat(200));
    let input = format!("a {{\n{line}\n}}");
    assert_error!(&input, expected(2, 6 + 64 + 1, &line));

    // Blocks: the 65th block body is one level too deep. The error is reported at its `{`.
    let input = format!("{}{}", "a {\n".repeat(200), "}\n".repeat(200));
    assert_error!(&input, expected(65, 3, "a {"));
}

#[test]
fn max_operator_depth_exceeded() {
    fn expected(line: usize, column: usize, line_content: &str) -> String {
        let message = "invalid binary operation; expected at most 128 chained binary operators";
        expected_error(line, column, line_content, message)
    }

    // The error is reported at the 129th operator. `x = 1` takes 5 columns, followed by
    // ` + 1` repeated, with the `+` in the second column of each repetition.
    let input = format!("x = 1{}", " + 1".repeat(200));
    assert_error!(&input, expected(1, 5 + 128 * 4 + 2, &input));

    // Operators of a chain within the first operand of another chain add up. The parenthesis
    // contains 100 operators, so the error is reported at the 29th operator after it. `x = (1`
    // takes 6 columns, followed by 100 ` + 1` and the closing parenthesis.
    let input = format!("x = (1{}){}", " + 1".repeat(100), " + 1".repeat(100));
    assert_error!(&input, expected(1, 6 + 100 * 4 + 1 + 28 * 4 + 2, &input));

    // Operators of sibling chains do not add up: both array elements are within the limit, so
    // the error is only reported at the 129th operator of the second element.
    let line = format!("x = [1{}, 1{}]", " + 1".repeat(128), " + 1".repeat(200));
    let input = format!("y = 1\n{line}");
    assert_error!(&input, expected(2, 6 + 128 * 4 + 3 + 128 * 4 + 2, &line));
}
