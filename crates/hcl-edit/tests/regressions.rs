use hcl_edit::expr::Expression;

// https://github.com/martinohmann/hcl-rs/issues/248
#[test]
fn issue_248() {
    let expr = Expression::from("${foo}");

    let encoded = expr.to_string();
    assert_eq!(encoded, "\"$${foo}\"");

    let parsed: Expression = encoded.parse().unwrap();
    assert_eq!(parsed, expr);
}
