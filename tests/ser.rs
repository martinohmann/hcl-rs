mod common;

use common::assert_serialize;
use hcl::expr::{Expression, RawExpression};
use hcl::structure::Attribute;
use indoc::indoc;

#[test]
fn custom_struct() {
    #[derive(serde::Serialize)]
    struct Test {
        foo: u32,
        bar: bool,
    }

    assert_serialize(
        Test { foo: 1, bar: true },
        indoc! {r#"
            foo = 1
            bar = true
        "#},
    );
}

#[test]
fn custom_tuple_struct() {
    #[derive(serde::Serialize)]
    struct Test1 {
        foo: u32,
    }

    #[derive(serde::Serialize)]
    struct Test2 {
        bar: &'static str,
    }

    #[derive(serde::Serialize)]
    struct TupleStruct(Test1, Test2);

    assert_serialize(
        TupleStruct(Test1 { foo: 1 }, Test2 { bar: "baz" }),
        indoc! {r#"
            foo = 1
            bar = "baz"
        "#},
    );
}

#[test]
fn custom_enum() {
    #[derive(serde::Serialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    #[derive(serde::Serialize, PartialEq, Debug)]
    struct Test {
        value: E,
    }

    assert_serialize(Test { value: E::Unit }, "value = \"Unit\"\n");
    assert_serialize(E::Newtype(1), "Newtype = 1\n");
    assert_serialize(
        E::Tuple(1, 2),
        indoc! {r#"
            Tuple = [
              1,
              2
            ]
        "#},
    );
    assert_serialize(
        Test {
            value: E::Struct { a: 1 },
        },
        indoc! {r#"
            value = {
              "Struct" = {
                "a" = 1
              }
            }
        "#},
    );
}

#[test]
fn body() {
    let value = hcl::body!({
        foo = 1
        bar = "baz"

        qux {
          foo = "bar"

          with_labels label1 "lab\"el2" {
            baz = [
              1,
              2,
              3
            ]
          }

          an_object = {
            foo = "bar"
            "enabled" = (RawExpression::new("var.enabled"))
            (RawExpression::from("var.name")) = "the value"
          }
        }
    });

    let expected = indoc! {r#"
        foo = 1
        bar = "baz"

        qux {
          foo = "bar"

          with_labels label1 "lab\"el2" {
            baz = [
              1,
              2,
              3
            ]
          }

          an_object = {
            foo = "bar"
            "enabled" = var.enabled
            var.name = "the value"
          }
        }
    "#};

    assert_serialize(value, expected);
}

#[test]
fn object() {
    let value = hcl::value!({
        foo = [1, 2, 3]
        bar = "baz"
        qux = { "foo" = "bar", "baz" = "qux" }
    });

    let expected = indoc! {r#"
        foo = [
          1,
          2,
          3
        ]
        bar = "baz"
        qux = {
          "foo" = "bar"
          "baz" = "qux"
        }
    "#};

    assert_serialize(value, expected);
}

#[test]
fn array() {
    let value = hcl::value!([
        { foo = [1, 2, 3] },
        { bar = "baz" },
        { qux = { "foo" = "bar", "baz" = "qux" } }
    ]);

    let expected = indoc! {r#"
        foo = [
          1,
          2,
          3
        ]
        bar = "baz"
        qux = {
          "foo" = "bar"
          "baz" = "qux"
        }
    "#};

    assert_serialize(value, expected);
}

#[test]
fn invalid_top_level_types() {
    assert!(hcl::to_string(&true).is_err());
    assert!(hcl::to_string("foo").is_err());
    assert!(hcl::to_string(&hcl::value!({ "\"" = "invalid attribute name" })).is_err())
}

#[test]
fn identifiers_with_hyphens() {
    assert_serialize(
        Attribute::new("hyphen-ated", Expression::Null),
        "hyphen-ated = null\n",
    );
}
