use super::Rule;
use crate::{Error, Result};
use indexmap::IndexMap as Map;
use pest::iterators::{Pair, Pairs};
use pest::Span;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub enum Node<'a> {
    Empty,
    Null(Pair<'a, Rule>),
    Boolean(Pair<'a, Rule>),
    Int(Pair<'a, Rule>),
    Float(Pair<'a, Rule>),
    String(Pair<'a, Rule>),
    Expression(Pair<'a, Rule>),
    Seq(Vec<Node<'a>>),
    Map(Map<String, Node<'a>>),
    Attribute(Map<String, Node<'a>>),
    Block(Map<String, Node<'a>>),
    BlockBody(Vec<Node<'a>>),
}

impl<'a> Node<'a> {
    /// Create a new `Node` from a `Pair`.
    pub fn from_pair(pair: Pair<'a, Rule>) -> Self {
        match pair.as_rule() {
            Rule::BooleanLit => Node::Boolean(pair),
            Rule::Float => Node::Float(pair),
            Rule::Heredoc => Node::String(pair.into_inner().nth(1).unwrap()),
            Rule::Identifier => Node::String(pair),
            Rule::Int => Node::Int(pair),
            Rule::NullLit => Node::Null(pair),
            Rule::StringLit => Node::String(pair.into_inner().next().unwrap()),
            Rule::Tuple => Node::Seq(collect_seq(pair)),
            Rule::BlockBody => Node::BlockBody(collect_seq(pair)),
            Rule::Object => Node::Map(collect_map(pair)),
            Rule::Attribute => Node::Attribute(collect_map(pair)),
            Rule::Block | Rule::BlockLabeled => Node::Block(collect_map(pair)),
            Rule::Body => {
                Node::Map(pair.into_inner().fold(Map::new(), |mut body, pair| {
                    let node = Node::from_pair(pair);
                    // We need to account for blocks with the same name and merge their contents.
                    //
                    // See: https://github.com/hashicorp/hcl/blob/main/json/spec.md#blocks
                    match node {
                        Node::Attribute(map) => map.into_iter().for_each(|(key, node)| {
                            body.insert(key, node);
                        }),
                        Node::Block(map) => map.into_iter().for_each(|(key, mut node)| {
                            body.entry(key)
                                .and_modify(|entry| entry.deep_merge_blocks(&mut node))
                                .or_insert(node);
                        }),
                        node => panic!("encountered unexpected node `{:?}`", node),
                    };

                    body
                }))
            }
            _ => Node::Expression(pair),
        }
    }

    /// Takes the value out of the `Node` and puts `Node::Empty` in its place.
    pub fn take(&mut self) -> Node<'a> {
        std::mem::replace(self, Node::Empty)
    }

    /// Returns a `Span` for the position of the `Node` in the parsed input, if available.
    pub fn as_span(&self) -> Option<Span<'a>> {
        self.as_pair().map(|pair| pair.as_span())
    }

    fn as_pair(&self) -> Option<&Pair<'a, Rule>> {
        match self {
            Node::Empty => None,
            Node::Null(pair) => Some(pair),
            Node::Boolean(pair) => Some(pair),
            Node::String(pair) => Some(pair),
            Node::Float(pair) => Some(pair),
            Node::Int(pair) => Some(pair),
            Node::Expression(pair) => Some(pair),
            Node::Seq(seq) | Node::BlockBody(seq) => seq.first().and_then(|n| n.as_pair()),
            Node::Map(map) | Node::Attribute(map) | Node::Block(map) => {
                map.first().and_then(|(_, n)| n.as_pair())
            }
        }
    }

    fn deep_merge_blocks(&mut self, other: &mut Node<'a>) {
        match (self, other) {
            (Node::Block(lhs), Node::Block(rhs)) => {
                rhs.iter_mut().for_each(|(key, node)| {
                    lhs.entry(key.to_string())
                        .and_modify(|lhs| lhs.deep_merge_blocks(node))
                        .or_insert_with(|| node.take());
                });
            }
            (Node::BlockBody(lhs), Node::BlockBody(rhs)) => {
                lhs.append(rhs);
            }
            (_, Node::Empty) => (),
            (lhs, rhs) => *lhs = rhs.take(),
        }
    }
}

fn collect_seq(pair: Pair<Rule>) -> Vec<Node> {
    pair.into_inner().map(Node::from_pair).collect()
}

fn collect_map(pair: Pair<Rule>) -> Map<String, Node> {
    KeyValueIter::new(pair).collect()
}

fn interpolate(s: &str) -> String {
    format!("${{{}}}", s)
}

struct KeyValueIter<'a> {
    inner: Pairs<'a, Rule>,
}

impl<'a> KeyValueIter<'a> {
    fn new(pair: Pair<'a, Rule>) -> Self {
        KeyValueIter {
            inner: pair.into_inner(),
        }
    }
}

impl<'a> Iterator for KeyValueIter<'a> {
    type Item = (String, Node<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.inner.next(), self.inner.next()) {
            (Some(k), Some(v)) => {
                let key = match Node::from_pair(k) {
                    Node::Expression(pair) => interpolate(pair.as_str()),
                    node => node
                        .as_pair()
                        .map(|pair| pair.as_str().to_owned())
                        .expect("failed to convert node to map key"),
                };

                Some((key, Node::from_pair(v)))
            }
            (Some(k), None) => panic!("missing node for key: {}", k),
            (_, _) => None,
        }
    }
}

impl<'a> TryFrom<Node<'a>> for () {
    type Error = Error;

    fn try_from(node: Node<'a>) -> Result<Self, Self::Error> {
        match node {
            Node::Null(_) => Ok(()),
            node => Err(Error::expected_span("null", node.as_span())),
        }
    }
}

impl<'a> TryFrom<Node<'a>> for bool {
    type Error = Error;

    fn try_from(node: Node<'a>) -> Result<Self, Self::Error> {
        match node {
            Node::Boolean(pair) => Ok(pair.as_str().parse().unwrap()),
            node => Err(Error::expected_span("boolean", node.as_span())),
        }
    }
}

impl<'a> TryFrom<Node<'a>> for char {
    type Error = Error;

    fn try_from(node: Node<'a>) -> Result<Self, Self::Error> {
        let span = node.as_span();

        match node {
            Node::String(pair) => {
                let mut chars = pair.as_str().chars();

                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok(c),
                    (_, _) => Err(Error::expected_span("char", span)),
                }
            }
            _ => Err(Error::expected_span("string", span)),
        }
    }
}

impl<'a> TryFrom<Node<'a>> for Cow<'a, str> {
    type Error = Error;

    fn try_from(node: Node<'a>) -> Result<Self, Self::Error> {
        match node {
            Node::String(pair) => Ok(pair.as_str().into()),
            Node::Expression(pair) => Ok(interpolate(pair.as_str()).into()),
            node => Err(Error::expected_span("string", node.as_span())),
        }
    }
}

impl<'a> TryFrom<Node<'a>> for Vec<Node<'a>> {
    type Error = Error;

    fn try_from(node: Node<'a>) -> Result<Self, Self::Error> {
        match node {
            Node::Seq(seq) | Node::BlockBody(seq) => Ok(seq),
            node => Err(Error::expected_span("sequence", node.as_span())),
        }
    }
}

impl<'a> TryFrom<Node<'a>> for Map<String, Node<'a>> {
    type Error = Error;

    fn try_from(node: Node<'a>) -> Result<Self, Self::Error> {
        match node {
            Node::Map(map) | Node::Attribute(map) | Node::Block(map) => Ok(map),
            node => Err(Error::expected_span("map", node.as_span())),
        }
    }
}

macro_rules! impl_try_from_int {
    ($($ty:ty),*) => {
        $(
            impl<'a> TryFrom<Node<'a>> for $ty {
                type Error = Error;

                fn try_from(node: Node<'a>) -> Result<Self, Self::Error> {
                    let span = node.as_span();

                    match node {
                        Node::Int(pair) => pair
                            .as_str()
                            .parse()
                            .map_err(|_| Error::new_span("Invalid int", span)),
                        _ => Err(Error::expected_span("int", span)),
                    }
                }
            }
        )*
    };
}

macro_rules! impl_try_from_float {
    ($($ty:ty),*) => {
        $(
            impl<'a> TryFrom<Node<'a>> for $ty {
                type Error = Error;

                fn try_from(node: Node<'a>) -> Result<Self, Self::Error> {
                    let span = node.as_span();

                    match node {
                        Node::Float(pair) => pair
                            .as_str()
                            .parse()
                            .map_err(|_| Error::new_span("Invalid float", span)),
                        _ => Err(Error::expected_span("float", span)),
                    }
                }
            }
        )*
    };
}

impl_try_from_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
impl_try_from_float!(f32, f64);
