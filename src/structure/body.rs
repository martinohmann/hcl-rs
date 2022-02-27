use super::{Attribute, Block, IntoNodeMap, Structure};
use crate::Value;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Body(Vec<Structure>);

impl Body {
    pub fn new() -> Body {
        Body::default()
    }

    pub fn into_inner(self) -> Vec<Structure> {
        self.0
    }

    pub fn builder() -> BodyBuilder {
        BodyBuilder::default()
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

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Default)]
pub struct BodyBuilder(Vec<Structure>);

impl BodyBuilder {
    pub fn add_attribute<A>(self, attr: A) -> BodyBuilder
    where
        A: Into<Attribute>,
    {
        self.add_structure(attr.into())
    }

    pub fn add_block<B>(self, block: B) -> BodyBuilder
    where
        B: Into<Block>,
    {
        self.add_structure(block.into())
    }

    pub fn add_structure<S>(mut self, structure: S) -> BodyBuilder
    where
        S: Into<Structure>,
    {
        self.0.push(structure.into());
        self
    }

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
