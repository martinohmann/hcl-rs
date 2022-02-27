//! Types to represent HCL structures.

mod attribute;
mod block;
mod body;

pub use self::attribute::Attribute;
pub use self::block::{Block, BlockBuilder, BlockLabel};
pub use self::body::{Body, BodyBuilder};
use crate::{Map, Value};

/// Represents an HCL structure.
///
/// There are two possible structures that can occur in an HCL [`Body`]: [`Attribute`]s and [`Block`]s.
#[derive(Debug, PartialEq, Clone)]
pub enum Structure {
    /// Represents an HCL attribute.
    Attribute(Attribute),
    /// Represents an HCL block.
    Block(Block),
}

impl From<Structure> for Value {
    fn from(s: Structure) -> Value {
        match s {
            Structure::Attribute(attr) => attr.into(),
            Structure::Block(block) => block.into(),
        }
    }
}

impl From<Attribute> for Structure {
    fn from(attr: Attribute) -> Structure {
        Structure::Attribute(attr)
    }
}

impl From<Block> for Structure {
    fn from(block: Block) -> Structure {
        Structure::Block(block)
    }
}

// A trait to convert an HCL structure into a map of nodes.
//
// This is used internally by the `Body` and `Block` types to convert into a `Value`.
//
// The detour over a map of nodes is necessary as HCL blocks with the same identifier and labels
// need to be merged so that the resulting `Value` conforms to the [HCL JSON
// specification](hcl-json-spec).
//
// [hcl-json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md#blocks
trait IntoNodeMap {
    fn into_node_map(self) -> Map<String, Node>;
}

impl IntoNodeMap for Body {
    fn into_node_map(self) -> Map<String, Node> {
        self.into_iter().fold(Map::new(), |mut map, structure| {
            match structure {
                Structure::Attribute(attr) => {
                    map.insert(attr.key, Node::Value(attr.value));
                }
                Structure::Block(block) => {
                    block
                        .into_node_map()
                        .into_iter()
                        .for_each(|(key, mut node)| {
                            map.entry(key)
                                .and_modify(|entry| entry.deep_merge(&mut node))
                                .or_insert(node);
                        });
                }
            };

            map
        })
    }
}

impl IntoNodeMap for Block {
    fn into_node_map(self) -> Map<String, Node> {
        let mut labels = self.labels.into_iter();

        let node = match labels.next() {
            Some(label) => {
                let block = Block {
                    identifier: label.into_inner(),
                    labels: labels.collect(),
                    body: self.body,
                };

                Node::Block(block.into_node_map())
            }
            None => Node::BlockInner(vec![self.body]),
        };

        Map::from_iter(std::iter::once((self.identifier, node)))
    }
}

enum Node {
    Empty,
    Block(Map<String, Node>),
    BlockInner(Vec<Body>),
    Value(Value),
}

impl From<Node> for Value {
    fn from(node: Node) -> Value {
        match node {
            Node::Empty => Value::Null,
            Node::Block(map) => Value::from_iter(map),
            Node::BlockInner(mut vec) => {
                // Flatten as per the [HCL JSON spec](json-spec).
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
            Node::Value(value) => value,
        }
    }
}

impl Node {
    fn take(&mut self) -> Node {
        std::mem::replace(self, Node::Empty)
    }

    fn deep_merge(&mut self, other: &mut Node) {
        match (self, other) {
            (Node::Block(lhs), Node::Block(rhs)) => {
                rhs.iter_mut().for_each(|(key, node)| {
                    lhs.entry(key.to_string())
                        .and_modify(|lhs| lhs.deep_merge(node))
                        .or_insert_with(|| node.take());
                });
            }
            (Node::BlockInner(lhs), Node::BlockInner(rhs)) => {
                lhs.append(rhs);
            }
            (lhs, rhs) => *lhs = rhs.take(),
        }
    }
}
