use super::*;
use pretty_assertions::assert_eq;

#[test]
fn expr_macro_primitives() {
    assert_eq!(expr!(null), Expression::Null);
    assert_eq!(expr!(true), Expression::Bool(true));
    assert_eq!(expr!(false), Expression::Bool(false));
    assert_eq!(expr!(0), Expression::Number(Number::from(0)));
    assert_eq!(expr!(1.5), Expression::Number(Number::from(1.5)));
    assert_eq!(expr!("foo"), Expression::String("foo".into()));
}

#[test]
fn expr_macro_arrays() {
    assert_eq!(
        expr!(["foo", 42]),
        Expression::Array(vec![
            Expression::String("foo".into()),
            Expression::Number(Number::from(42))
        ])
    );
}

#[test]
fn expr_macro_objects() {
    let expected = Expression::Object({
        let mut object = Object::new();
        object.insert("foo".into(), "bar".into());
        object.insert("baz".into(), true.into());
        object.insert("qux".into(), vec![1, 2, 3].into());
        object
    });

    assert_eq!(
        expr!({
            "foo" = "bar",
            "baz" = true,
            "qux" = [1, 2, 3]
        }),
        expected
    );
}

#[test]
fn attr_macro() {
    assert_eq!(
        attr!(foo = {}),
        Attribute::new("foo", Expression::Object(Object::new()))
    );

    let foo = "bar";

    assert_eq!(
        attr!((foo) = {}),
        Attribute::new("bar", Expression::Object(Object::new()))
    );
}

#[test]
fn block_macro() {
    assert_eq!(block!(foo {}), Block::builder("foo").build());
    assert_eq!(
        block!(resource "aws_s3_bucket" "bucket" {}),
        Block::builder("resource")
            .add_label(BlockLabel::string("aws_s3_bucket"))
            .add_label(BlockLabel::string("bucket"))
            .build()
    );

    assert_eq!(
        block!(resource aws_s3_bucket bucket {}),
        Block::builder("resource")
            .add_label(BlockLabel::identifier("aws_s3_bucket"))
            .add_label(BlockLabel::identifier("bucket"))
            .build()
    );

    let ident = "resource";
    let name = "bucket";

    assert_eq!(
        block!((ident) aws_s3_bucket (name) {}),
        Block::builder("resource")
            .add_label(BlockLabel::identifier("aws_s3_bucket"))
            .add_label(BlockLabel::string("bucket"))
            .build()
    );
}

#[test]
fn body_macro() {
    assert_eq!(body!({}), Body::builder().build());
    assert_eq!(
        body!({
            foo = "bar"
            baz = "qux"
            qux "foo" {
                bar = 42
            }
        }),
        Body::builder()
            .add_attribute(("foo", "bar"))
            .add_attribute(("baz", "qux"))
            .add_block(
                Block::builder("qux")
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
}
