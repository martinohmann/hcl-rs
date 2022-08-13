use super::{
    attribute::AttributeSerializer,
    block::{BlockLabelSerializer, BlockSerializer},
    expression::ExpressionSerializer,
    *,
};
use crate::{Attribute, Block, BlockLabel, Expression};
use std::fmt::Debug;

#[track_caller]
fn test_identity<S, T>(ser: S, value: T)
where
    S: ser::Serializer<Ok = T>,
    T: ser::Serialize + PartialEq + Debug,
{
    assert_eq!(value, value.serialize(ser).unwrap());
}

#[test]
fn identity() {
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
    test_identity(BlockLabelSerializer, BlockLabel::string("foo"));
    test_identity(BlockLabelSerializer, BlockLabel::identifier("foo"));
    test_identity(ExpressionSerializer, Expression::String("bar".into()));
    test_identity(
        ExpressionSerializer,
        Expression::from_iter([("foo", "bar")]),
    );
}
