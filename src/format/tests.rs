use super::*;
use crate::{Attribute, Expression, FuncCall};

#[test]
fn issue_87() {
    let expr = Expression::from(
        FuncCall::builder("foo")
            .arg(Expression::from_iter([("bar", FuncCall::new("baz"))]))
            .build(),
    );

    let result = to_string(&expr).unwrap();

    assert_eq!(result, "foo({\"bar\" = baz()})")
}

#[test]
fn issue_91() {
    let attr = Attribute::new("_foo", "bar");
    let result = to_string(&attr).unwrap();

    assert_eq!(result, "_foo = \"bar\"\n")
}
