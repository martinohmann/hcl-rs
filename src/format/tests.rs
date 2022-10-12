use super::*;
use crate::{Expression, FuncCall};

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
