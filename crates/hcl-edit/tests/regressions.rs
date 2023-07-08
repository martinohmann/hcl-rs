use hcl_edit::expr::Expression;
use hcl_edit::template::{Element, Interpolation, Template};
use hcl_edit::Ident;
use pretty_assertions::assert_eq;

// https://github.com/martinohmann/hcl-rs/issues/248
#[test]
fn issue_248() {
    let expr = Expression::from("${foo}");

    let encoded = expr.to_string();
    assert_eq!(encoded, "\"$${foo}\"");

    let parsed: Expression = encoded.parse().unwrap();
    assert_eq!(parsed, expr);
}

// https://github.com/martinohmann/hcl-rs/issues/256
#[test]
fn issue_256() {
    let input = "$${escaped1} ${unescaped} $${escaped2} $$ESCAPED_SHELL_VAR\n$SHELL_VAR";
    let parsed: Template = input.parse().unwrap();
    let expected = Template::from_iter([
        Element::from("${escaped1} "),
        Element::from(Interpolation::new(Ident::new("unescaped"))),
        Element::from(" ${escaped2} $$ESCAPED_SHELL_VAR\n$SHELL_VAR"),
    ]);

    assert_eq!(parsed, expected);
}
