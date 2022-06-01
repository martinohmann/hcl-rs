//! Types to represent and build HCL body structures.

use super::{Attribute, Block, IntoNodeMap, Structure};
use crate::{Map, Value};
use serde::Deserialize;
use std::vec::IntoIter;

/// Represents an HCL config file body.
///
/// A `Body` consists of zero or more [`Attribute`] and [`Block`] HCL structures.
#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
#[serde(rename = "$hcl::body")]
pub struct Body(pub Vec<Structure>);

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
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.0.iter(),
        }
    }

    /// Returns an iterator over all [`Structure`]s of the `Body` that allows modifying the
    /// structures.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            inner: self.0.iter_mut(),
        }
    }

    /// Returns an iterator over all [`Attribute`]s of the `Body`.
    pub fn attributes(&self) -> AttributeIter<'_> {
        AttributeIter { inner: self.iter() }
    }

    /// Returns an iterator over all [`Attribute`]s of the `Body` that allows modifying the
    /// attributes.
    pub fn attributes_mut(&mut self) -> AttributeIterMut<'_> {
        AttributeIterMut {
            inner: self.iter_mut(),
        }
    }

    /// Returns an iterator over all [`Block`]s of the `Body`.
    pub fn blocks(&self) -> BlockIter<'_> {
        BlockIter { inner: self.iter() }
    }

    /// Returns an iterator over all [`Block`]s of the `Body` that allows modifying the blocks.
    pub fn blocks_mut(&mut self) -> BlockIterMut<'_> {
        BlockIterMut {
            inner: self.iter_mut(),
        }
    }
}

/// Immutable body iterator.
///
/// This struct is created by the [`iter`](Body::iter) method on a [`Body`].
pub struct Iter<'a> {
    inner: std::slice::Iter<'a, Structure>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Structure;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Mutable body iterator.
///
/// This struct is created by the [`iter_mut`](Body::iter_mut) method on a [`Body`].
pub struct IterMut<'a> {
    inner: std::slice::IterMut<'a, Structure>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Structure;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Immutable `Attribute` iterator.
///
/// This struct is created by the [`attributes`](Body::attributes) method on a [`Body`].
pub struct AttributeIter<'a> {
    inner: Iter<'a>,
}

impl<'a> Iterator for AttributeIter<'a> {
    type Item = &'a Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            if let Structure::Attribute(attr) = structure {
                return Some(attr);
            }
        }

        None
    }
}

/// Mutable `Attribute` iterator.
///
/// This struct is created by the [`attributes_mut`](Body::attributes_mut) method on a [`Body`].
pub struct AttributeIterMut<'a> {
    inner: IterMut<'a>,
}

impl<'a> Iterator for AttributeIterMut<'a> {
    type Item = &'a mut Attribute;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            if let Structure::Attribute(attr) = structure {
                return Some(attr);
            }
        }

        None
    }
}

/// Immutable `Block` iterator.
///
/// This struct is created by the [`blocks`](Body::blocks) method on a [`Body`].
pub struct BlockIter<'a> {
    inner: Iter<'a>,
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            if let Structure::Block(block) = structure {
                return Some(block);
            }
        }

        None
    }
}

/// Mutable `Block` iterator.
///
/// This struct is created by the [`blocks_mut`](Body::blocks_mut) method on a [`Body`].
pub struct BlockIterMut<'a> {
    inner: IterMut<'a>,
}

impl<'a> Iterator for BlockIterMut<'a> {
    type Item = &'a mut Block;

    fn next(&mut self) -> Option<Self::Item> {
        for structure in &mut self.inner {
            if let Structure::Block(block) = structure {
                return Some(block);
            }
        }

        None
    }
}

impl From<Body> for Value {
    fn from(body: Body) -> Value {
        Value::Object(body.into())
    }
}

impl From<Body> for Map<String, Value> {
    fn from(body: Body) -> Map<String, Value> {
        body.into_node_map()
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
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

    /// Adds `Attribute`s to the body from an iterator.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_attributes<I>(self, iter: I) -> BodyBuilder
    where
        I: IntoIterator,
        I::Item: Into<Attribute>,
    {
        self.add_structures(iter.into_iter().map(Into::into))
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

    /// Adds `Block`s to the body from an iterator.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_blocks<I>(self, iter: I) -> BodyBuilder
    where
        I: IntoIterator,
        I::Item: Into<Block>,
    {
        self.add_structures(iter.into_iter().map(Into::into))
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

    /// Adds `Structure`s to the body from an iterator.
    ///
    /// Consumes `self` and returns a new `BodyBuilder`.
    pub fn add_structures<I>(mut self, iter: I) -> BodyBuilder
    where
        I: IntoIterator,
        I::Item: Into<Structure>,
    {
        self.0.extend(iter.into_iter().map(Into::into));
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
    use crate::RawExpression;
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
                    .add_attribute(("baz", RawExpression::new("var.foo")))
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
                        "bar": "baz",
                        "baz": "${var.foo}"
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
