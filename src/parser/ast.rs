use super::Rule;
use indexmap::IndexMap as Map;
use pest::iterators::{Pair, Pairs};
use pest::Span;

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
            Rule::Hcl | Rule::BlockBodyInner => {
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

    fn as_map_key(&self) -> String {
        match self {
            Node::Expression(pair) => interpolate(pair.as_str()),
            node => node
                .as_pair()
                .map(|pair| pair.as_str().to_owned())
                .expect("map key"),
        }
    }

    fn deep_merge_blocks(&mut self, other: &mut Node<'a>) {
        match (self, other) {
            (Node::Block(lhs), Node::Block(rhs)) => {
                rhs.iter_mut().for_each(|(key, value)| {
                    lhs.entry(key.to_string())
                        .and_modify(|lhs| lhs.deep_merge_blocks(value))
                        .or_insert_with(|| std::mem::replace(value, Node::Empty));
                });
            }
            (Node::BlockBody(lhs), Node::BlockBody(rhs)) => {
                lhs.append(rhs);
            }
            (_, Node::Empty) => (),
            (lhs, rhs) => *lhs = std::mem::replace(rhs, Node::Empty),
        }
    }
}

fn collect_seq(pair: Pair<Rule>) -> Vec<Node> {
    pair.into_inner().map(Node::from_pair).collect()
}

fn collect_map(pair: Pair<Rule>) -> Map<String, Node> {
    KeyValueIter::new(pair).collect()
}

pub fn interpolate(s: &str) -> String {
    if s.starts_with("${") {
        s.to_owned()
    } else {
        format!("${{{}}}", s)
    }
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
                let key = Node::from_pair(k).as_map_key();
                let node = Node::from_pair(v);
                Some((key, node))
            }
            (Some(k), None) => panic!("missing node for key: {}", k),
            (_, _) => None,
        }
    }
}
