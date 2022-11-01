use super::{block::BlockLabelSerializer, AttributeSerializer, BlockSerializer, BodySerializer};
use crate::structure::{Attribute, Block, BlockLabel, Body};
use crate::{Identifier, Map};
use serde::{ser, Serialize};
use std::fmt::Debug;

#[track_caller]
fn test_identity<S, T>(ser: S, value: T)
where
    S: ser::Serializer<Ok = T>,
    T: ser::Serialize + PartialEq + Debug,
{
    assert_eq!(value, value.serialize(ser).unwrap());
}

#[track_caller]
fn test_serialize<S, G, E>(ser: S, given: G, expected: E)
where
    S: ser::Serializer<Ok = E>,
    G: ser::Serialize,
    E: PartialEq + Debug,
{
    assert_eq!(expected, given.serialize(ser).unwrap());
}

#[test]
fn identity() {
    test_identity(BodySerializer, Body::default());
    test_identity(
        BodySerializer,
        Body::builder()
            .add_attribute(("foo", "bar"))
            .add_block(Block::builder("baz").build())
            .build(),
    );
    test_identity(AttributeSerializer, Attribute::new("foo", "bar"));
    test_identity(
        AttributeSerializer,
        Attribute::new("foo", vec!["bar", "baz"]),
    );
    test_identity(
        BlockSerializer,
        Block::builder("foo")
            .add_label("bar")
            .add_attribute(("baz", "qux"))
            .build(),
    );
    test_identity(BlockLabelSerializer, BlockLabel::from("foo"));
    test_identity(
        BlockLabelSerializer,
        BlockLabel::from(Identifier::unchecked("foo")),
    );
}

#[test]
fn custom() {
    #[derive(Serialize)]
    struct CustomAttr {
        key: &'static str,
        #[serde(rename = "expr")]
        value: &'static str,
    }
    test_serialize(
        AttributeSerializer,
        CustomAttr {
            key: "foo",
            value: "bar",
        },
        Attribute::new("foo", "bar"),
    );
    test_serialize(
        AttributeSerializer,
        ("foo", "bar"),
        Attribute::new("foo", "bar"),
    );

    test_serialize(
        BlockSerializer,
        {
            let mut map = Map::new();
            map.insert("foo", (("bar", "baz"), ("qux", "foo")));
            map
        },
        Block::builder("foo")
            .add_attribute(("bar", "baz"))
            .add_attribute(("qux", "foo"))
            .build(),
    );

    #[derive(Serialize)]
    struct CustomBlock {
        #[serde(rename = "identifier")]
        ident: &'static str,
        body: Map<&'static str, &'static str>,
    }

    test_serialize(
        BlockSerializer,
        CustomBlock {
            ident: "foo",
            body: {
                let mut map = Map::new();
                map.insert("bar", "baz");
                map.insert("qux", "foo");
                map
            },
        },
        Block::builder("foo")
            .add_attribute(("bar", "baz"))
            .add_attribute(("qux", "foo"))
            .build(),
    );

    #[derive(Serialize)]
    struct CustomLabeledBlock {
        #[serde(rename = "identifier")]
        ident: &'static str,
        labels: [&'static str; 2],
        body: Map<&'static str, &'static str>,
    }

    test_serialize(
        BlockSerializer,
        CustomLabeledBlock {
            ident: "foo",
            labels: ["bar", "baz"],
            body: {
                let mut map = Map::new();
                map.insert("qux", "foo");
                map
            },
        },
        Block::builder("foo")
            .add_labels(["bar", "baz"])
            .add_attribute(("qux", "foo"))
            .build(),
    );
}
