use crate::expr::{Expression, Object, ObjectKey, RawExpression};
use crate::structure::{Attribute, Block, Body, Structure};
use crate::{Identifier, Number};
use pretty_assertions::assert_eq;

#[test]
fn expression_macro_primitives() {
    assert_eq!(expression!(null), Expression::Null);
    assert_eq!(expression!(true), Expression::Bool(true));
    assert_eq!(expression!(false), Expression::Bool(false));
    assert_eq!(expression!(0), Expression::Number(Number::from(0)));
    assert_eq!(expression!(1.5), Expression::from(1.5));
    assert_eq!(expression!("foo"), Expression::String("foo".into()));
}

#[test]
fn expression_macro_arrays() {
    assert_eq!(
        expression!(["foo", 42]),
        Expression::Array(vec![
            Expression::String("foo".into()),
            Expression::Number(Number::from(42))
        ])
    );
}

#[test]
fn expression_macro_objects() {
    let expected = Expression::Object(Object::from([
        (ObjectKey::from("foo"), "bar".into()),
        (ObjectKey::from("baz"), true.into()),
        (ObjectKey::from("qux"), vec![1, 2, 3].into()),
        (ObjectKey::from(1), 2.into()),
    ]));

    assert_eq!(
        expression!({
            "foo" = "bar",
            "baz" = true,
            "qux" = [1, 2, 3],
            1 = 2
        }),
        expected
    );

    let expected = Expression::Object(Object::from([
        (ObjectKey::from(Identifier::unchecked("foo")), "bar".into()),
        (ObjectKey::from("bar"), true.into()),
        (
            ObjectKey::Expression(RawExpression::from("qux").into()),
            vec![1, 2, 3].into(),
        ),
    ]));

    let baz = "bar";

    assert_eq!(
        expression!({
            foo = (baz)
            (baz) = true
            #{"qux"} = [1, 2, 3]
        }),
        expected
    );
}

#[test]
fn attribute_macro() {
    assert_eq!(
        attribute!(foo = 1),
        Attribute::new("foo", Expression::Number(1.into()))
    );

    let foo = "bar";

    assert_eq!(
        attribute!((foo) = {}),
        Attribute::new("bar", Expression::Object(Object::new()))
    );
}

#[test]
fn block_macro() {
    assert_eq!(block!(foo {}), Block::builder("foo").build());
    assert_eq!(
        block!(resource "aws_s3_bucket" "bucket" {}),
        Block::builder("resource")
            .add_label("aws_s3_bucket")
            .add_label("bucket")
            .build()
    );

    assert_eq!(
        block!(resource aws_s3_bucket bucket {}),
        Block::builder("resource")
            .add_label(Identifier::unchecked("aws_s3_bucket"))
            .add_label(Identifier::unchecked("bucket"))
            .build()
    );

    let ident = "resource";
    let name = "bucket";

    assert_eq!(
        block!((ident) aws_s3_bucket (name) {}),
        Block::builder("resource")
            .add_label(Identifier::unchecked("aws_s3_bucket"))
            .add_label("bucket")
            .build()
    );
}

#[test]
fn body_macro() {
    assert_eq!(body!({}), Body::builder().build());
    let bar = "foo";
    assert_eq!(
        body!({
            foo = "bar"
            baz = "qux"
            qux "foo" (bar) {
                bar = 42
            }
        }),
        Body::builder()
            .add_attribute(("foo", "bar"))
            .add_attribute(("baz", "qux"))
            .add_block(
                Block::builder("qux")
                    .add_label("foo")
                    .add_label("foo")
                    .add_attribute(("bar", 42))
                    .build()
            )
            .build()
    );
}

#[test]
fn structure_macro() {
    let foo = "bar";
    assert_eq!(
        structure!(foo {}),
        Structure::Block(Block::builder("foo").build())
    );
    assert_eq!(
        structure!((foo) {}),
        Structure::Block(Block::builder("bar").build())
    );
    assert_eq!(
        structure!(foo = "bar"),
        Structure::Attribute(Attribute::new("foo", "bar"))
    );
    assert_eq!(
        structure!((foo) = "bar"),
        Structure::Attribute(Attribute::new("bar", "bar"))
    );
    assert_eq!(
        structure!((foo) = #{"raw"}),
        Structure::Attribute(Attribute::new("bar", RawExpression::new("raw")))
    );
}
