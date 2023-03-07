use crate::structure::{Attribute, Block, Body, Structure};
use crate::Identifier;
use pretty_assertions::assert_eq;

#[track_caller]
fn assert_body<T>(given: T, expected: Body)
where
    T: serde::Serialize,
{
    assert_eq!(Body::from_serializable(&given).unwrap(), expected);
}

#[test]
fn roundtrip() {
    assert_body(Body::default(), Body::default());
    assert_body(
        Body::builder()
            .add_attribute(("foo", "bar"))
            .add_block(Block::builder("baz").build())
            .build(),
        Body::builder()
            .add_attribute(("foo", "bar"))
            .add_block(Block::builder("baz").build())
            .build(),
    );
    assert_body(
        Structure::Block(
            Block::builder("foo")
                .add_label("bar")
                .add_label(Identifier::unchecked("bar"))
                .add_attribute(("baz", "qux"))
                .build(),
        ),
        Block::builder("foo")
            .add_label("bar")
            .add_label(Identifier::unchecked("bar"))
            .add_attribute(("baz", "qux"))
            .build()
            .into(),
    );
}

#[test]
fn builtin() {
    assert_body(
        Attribute::new("foo", "bar"),
        Attribute::new("foo", "bar").into(),
    );
    assert_body(
        Attribute::new("foo", vec!["bar", "baz"]),
        Attribute::new("foo", vec!["bar", "baz"]).into(),
    );
    assert_body(
        Block::builder("foo")
            .add_label("bar")
            .add_label(Identifier::unchecked("bar"))
            .add_attribute(("baz", "qux"))
            .build(),
        Block::builder("foo")
            .add_label("bar")
            .add_label(Identifier::unchecked("bar"))
            .add_attribute(("baz", "qux"))
            .build()
            .into(),
    );
}
