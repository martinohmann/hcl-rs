use super::*;
use crate::{Attribute, Expression, FuncCall};

#[track_caller]
fn expect_format<T: Format>(value: T, expected: &str) {
    assert_eq!(to_string(&value).unwrap(), expected);
}

#[test]
fn issue_87() {
    let expr = Expression::from(
        FuncCall::builder("foo")
            .arg(Expression::from_iter([("bar", FuncCall::new("baz"))]))
            .build(),
    );
    expect_format(expr, "foo({\"bar\" = baz()})");
}

#[test]
fn issue_91() {
    expect_format(Attribute::new("_foo", "bar"), "_foo = \"bar\"\n");
}
