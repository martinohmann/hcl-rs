//! Types to represent and build HCL body structures.

use super::{Attribute, Block, IntoNodeMap, Structure};
use crate::Value;
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;

/// Represents an HCL config file body.
///
/// A `Body` consists of zero or more [`Attribute`] and [`Block`] HCL structures.
#[derive(Debug, PartialEq, Default, Clone)]
pub struct Body(Vec<Structure>);

impl Body {
    /// Consumes `self` and returns the wrapped `Vec<Structure>`.
    pub fn into_inner(self) -> Vec<Structure> {
        self.0
    }

    /// Creates a new [`BodyBuilder`] to start building a new `Body`.
    pub fn builder() -> BodyBuilder {
        BodyBuilder::default()
    }

    /// Returns an iterator over all [`Structure`]s of the `Body`.
    pub fn iter(&self) -> Iter<'_, Structure> {
        self.0.iter()
    }

    /// Returns an iterator over all [`Structure`]s of the `Body` that allows modifying the
    /// structures.
    pub fn iter_mut(&mut self) -> IterMut<'_, Structure> {
        self.0.iter_mut()
    }
}

impl From<Body> for Value {
    fn from(body: Body) -> Value {
        Value::from_iter(body.into_node_map())
    }
}

impl<S> FromIterator<S> for Body
where
    S: Into<Structure>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = S>,
    {
        Body(iter.into_iter().map(Into::into).collect())
    }
}

impl IntoIterator for Body {
    type Item = Structure;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// `BodyBuilder` builds a HCL [`Body`].
///
/// The builder allows to build the `Body` by adding attributes and other nested blocks via chained
/// method calls. A call to [`.build()`](BodyBuilder::build) produces the final `Body`.
///
/// ## Example
///
/// ```
/// use hcl::{Body, Block};
///
/// let body = Body::builder()
///     .add_block(
///         Block::builder("resource")
///             .add_label("aws_s3_bucket")
///             .add_label("mybucket")
///             .add_attribute(("name", "mybucket"))
///             .build()
///     )
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct BodyBuilder(Vec<Structure>);

impl BodyBuilder {
    /// Adds an `Attribute` to the body.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_attribute<A>(self, attr: A) -> BodyBuilder
    where
        A: Into<Attribute>,
    {
        self.add_structure(attr.into())
    }

    /// Adds a `Block` to the body.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_block<B>(self, block: B) -> BodyBuilder
    where
        B: Into<Block>,
    {
        self.add_structure(block.into())
    }

    /// Adds a `Structure` to the body.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_structure<S>(mut self, structure: S) -> BodyBuilder
    where
        S: Into<Structure>,
    {
        self.0.push(structure.into());
        self
    }

    /// Consumes `self` and builds the [`Body`] from the structures added via the builder methods.
    pub fn build(self) -> Body {
        Body::from_iter(self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_into_value() {
        let body = Body::builder()
            .add_attribute(("foo", "bar"))
            .add_attribute(("bar", "baz"))
            .add_block(
                Block::builder("bar")
                    .add_label("baz")
                    .add_attribute(("foo", "bar"))
                    .build(),
            )
            .add_block(
                Block::builder("bar")
                    .add_label("qux")
                    .add_attribute(("foo", 1))
                    .build(),
            )
            .add_block(
                Block::builder("bar")
                    .add_label("baz")
                    .add_attribute(("bar", "baz"))
                    .build(),
            )
            .add_attribute(("foo", "baz"))
            .build();

        let value = json!({
            "foo": "baz",
            "bar": {
                "baz": [
                    {
                        "foo": "bar"
                    },
                    {
                        "bar": "baz"
                    }
                ],
                "qux": {
                    "foo": 1
                }
            }
        });

        let expected: Value = serde_json::from_value(value).unwrap();

        assert_eq!(Value::from(body), expected);
    }
}
