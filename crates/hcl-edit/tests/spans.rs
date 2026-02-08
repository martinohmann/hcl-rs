use hcl_edit::parser::parse_expr;
use pretty_assertions::assert_eq;

macro_rules! assert_span {
    ($expr:expr, $span:expr) => {
        assert_eq!(hcl_edit::Span::span(&$expr), Some($span));
    };
}

#[test]
fn number() {
    let expr = parse_expr("42").unwrap();
    assert_span!(expr, 0..2);
}

#[test]
fn binary_op() {
    let expr = parse_expr("1 + 2").unwrap();
    assert_span!(expr, 0..5);

    let binary_op = expr.as_binary_op().unwrap();
    assert_span!(binary_op.lhs_expr, 0..1);
    assert_span!(binary_op.operator, 2..3);
    assert_span!(binary_op.rhs_expr, 4..5);
}

#[test]
fn binary_ops() {
    let expr = parse_expr("1 + 2 + 3 * 4").unwrap();
    assert_span!(expr, 0..13);

    let binary_op = expr.as_binary_op().unwrap();
    assert_span!(binary_op.lhs_expr, 0..5);
    assert_span!(binary_op.operator, 6..7);
    assert_span!(binary_op.rhs_expr, 8..13);

    let lhs = binary_op.lhs_expr.as_binary_op().unwrap();
    assert_span!(lhs.lhs_expr, 0..1);
    assert_span!(lhs.operator, 2..3);
    assert_span!(lhs.rhs_expr, 4..5);

    let rhs = binary_op.rhs_expr.as_binary_op().unwrap();
    assert_span!(rhs.lhs_expr, 8..9);
    assert_span!(rhs.operator, 10..11);
    assert_span!(rhs.rhs_expr, 12..13);
}

#[test]
fn unary_op() {
    let expr = parse_expr("! true").unwrap();
    assert_span!(expr, 0..6);

    let unary_op = expr.as_unary_op().unwrap();
    assert_span!(unary_op.operator, 0..1);
    assert_span!(unary_op.expr, 2..6);
}

#[test]
fn conditional() {
    let expr = parse_expr("true ? 1 : 2").unwrap();
    assert_span!(expr, 0..12);

    let conditional = expr.as_conditional().unwrap();
    assert_span!(conditional.cond_expr, 0..4);
    assert_span!(conditional.true_expr, 7..8);
    assert_span!(conditional.false_expr, 11..12);
}

#[test]
fn traversal() {
    let expr = parse_expr("foo.bar [0]").unwrap();
    assert_span!(expr, 0..11);

    let traversal = expr.as_traversal().unwrap();
    assert_span!(traversal.expr, 0..3);
    assert_span!(traversal.operators[0], 3..7);
    assert_span!(traversal.operators[1], 8..11);
}
