use super::*;
use crate::expr::{Expression, FuncCall};
use crate::structure::Attribute;

#[track_caller]
fn expect_format<T: Format>(value: T, expected: &str) {
    assert_eq!(to_string(&value).unwrap(), expected);
}

fn expect_formatb<'a, F, T>(f: F, value: T, expected: &str)
where
    T: Format,
    F: FnOnce(FormatterBuilder<'a, &'a mut Vec<u8>>) -> FormatterBuilder<'a, &'a mut Vec<u8>>,
{
    let mut buf = Vec::with_capacity(128);
    let mut fmt = f(Formatter::builder()).build(&mut buf);
    value.format(&mut fmt).unwrap();
    let formatted = std::str::from_utf8(&buf).unwrap();
    assert_eq!(formatted, expected);
}

#[test]
fn issue_87() {
    let expr = Expression::from(
        FuncCall::builder("foo")
            .arg(Expression::from_iter([("bar", FuncCall::new("baz"))]))
            .build(),
    );
    expect_format(expr, "foo({ \"bar\" = baz() })");
}

#[test]
fn issue_91() {
    expect_format(Attribute::new("_foo", "bar"), "_foo = \"bar\"\n");
}

#[test]
fn compact_func_args() {
    expect_format(
        FuncCall::builder("func")
            .arg(vec![1, 2, 3])
            .arg(expression!({
                foo = "bar"
                baz = "qux"
            }))
            .build(),
        "func([1, 2, 3], { foo = \"bar\", baz = \"qux\" })",
    );
}

#[test]
fn compact_arrays() {
    let attr = Attribute::new("array", expression!([1, 2, 3, [4, 5]]));

    expect_formatb(
        |b| b.compact_arrays(true),
        attr,
        "array = [1, 2, 3, [4, 5]]\n",
    );
}

#[test]
fn compact_objects() {
    let attr = Attribute::new(
        "object",
        expression!({
            foo = {
                bar = "baz"
            }
            qux = "bam"
        }),
    );

    expect_formatb(
        |b| b.compact_objects(true),
        attr,
        "object = { foo = { bar = \"baz\" }, qux = \"bam\" }\n",
    );
}
