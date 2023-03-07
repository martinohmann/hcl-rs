use super::{Block, Body, Structure};
use crate::{Expression, Identifier, Map, Value};
use indexmap::map::Entry;

/// A trait to convert an HCL structure into its [JSON representation][json-spec].
///
/// This is used internally by the `Body` and `Block` types to convert into an `Expression`.
///
/// [json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md#blocks
pub(crate) trait IntoJsonSpec: Sized {
    /// Converts a value to an expression that conforms to the HCL JSON specification.
    ///
    /// Provides a default implementation which converts the result of `into_nodes` into an
    /// `Expression` and unsually does not need to be overridden.
    fn into_json_spec(self) -> Expression {
        Expression::from_iter(self.into_json_nodes())
    }

    /// Converts the value into a map of nodes.
    ///
    /// The detour over a map of nodes is necessary as HCL blocks with the same identifier and
    /// labels need to be merged so that the `Expression` resulting from `into_json_spec` conforms
    /// to the HCL JSON specification.
    fn into_json_nodes(self) -> Map<String, JsonNode>;
}

impl IntoJsonSpec for Body {
    fn into_json_nodes(self) -> Map<String, JsonNode> {
        self.into_iter().fold(Map::new(), |mut map, structure| {
            match structure {
                Structure::Attribute(attr) => {
                    map.insert(attr.key.into_inner(), JsonNode::Expr(attr.expr));
                }
                Structure::Block(block) => {
                    for (key, node) in block.into_json_nodes() {
                        node.deep_merge_into(&mut map, key);
                    }
                }
            };

            map
        })
    }
}

impl IntoJsonSpec for Block {
    fn into_json_nodes(self) -> Map<String, JsonNode> {
        let mut labels = self.labels.into_iter();

        let node = match labels.next() {
            Some(label) => {
                let block = Block {
                    identifier: Identifier::unchecked(label.into_inner()),
                    labels: labels.collect(),
                    body: self.body,
                };

                JsonNode::Map(block.into_json_nodes())
            }
            None => JsonNode::Body(vec![self.body]),
        };

        std::iter::once((self.identifier.into_inner(), node)).collect()
    }
}

pub(crate) enum JsonNode {
    Map(Map<String, JsonNode>),
    Body(Vec<Body>),
    Expr(Expression),
}

impl From<JsonNode> for Expression {
    fn from(node: JsonNode) -> Self {
        match node {
            JsonNode::Map(map) => Expression::from_iter(map),
            JsonNode::Body(mut vec) => {
                // Flatten as per the [HCL JSON spec][json-spec].
                //
                // > After any labelling levels, the next nested value is either a JSON
                // > object representing a single block body, or a JSON array of JSON
                // > objects that each represent a single block body.
                //
                // [json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md#blocks
                if vec.len() == 1 {
                    vec.remove(0).into()
                } else {
                    vec.into()
                }
            }
            JsonNode::Expr(expr) => expr,
        }
    }
}

impl<T> From<T> for Expression
where
    T: IntoJsonSpec,
{
    fn from(value: T) -> Expression {
        value.into_json_spec()
    }
}

impl<T> From<T> for Value
where
    T: IntoJsonSpec,
{
    fn from(value: T) -> Value {
        Value::from(value.into_json_spec())
    }
}

impl JsonNode {
    fn deep_merge_into(self, map: &mut Map<String, JsonNode>, key: String) {
        match map.entry(key) {
            Entry::Occupied(o) => o.into_mut().deep_merge(self),
            Entry::Vacant(v) => {
                v.insert(self);
            }
        }
    }

    fn deep_merge(&mut self, other: JsonNode) {
        match (self, other) {
            (JsonNode::Map(lhs), JsonNode::Map(rhs)) => {
                for (key, node) in rhs {
                    node.deep_merge_into(lhs, key);
                }
            }
            (JsonNode::Body(lhs), JsonNode::Body(mut rhs)) => {
                lhs.append(&mut rhs);
            }
            (lhs, rhs) => *lhs = rhs,
        }
    }
}
