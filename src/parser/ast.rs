use super::Rule;
use indexmap::{map::Entry, IndexMap as Map};
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
}

impl<'a> Node<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>) -> Self {
        match pair.as_rule() {
            Rule::boolean_lit => Node::Boolean(pair),
            Rule::float => Node::Float(pair),
            Rule::heredoc => Node::String(pair.into_inner().nth(1).unwrap()),
            Rule::identifier => Node::String(pair),
            Rule::int => Node::Int(pair),
            Rule::null_lit => Node::Null(pair),
            Rule::string_lit => Node::String(pair.into_inner().next().unwrap()),
            Rule::tuple | Rule::block_body => {
                Node::Seq(pair.into_inner().map(Node::from_pair).collect())
            }
            Rule::attribute | Rule::object => {
                let mut map = Map::new();
                overwrite_nodes(&mut map, pair.into_inner());
                Node::Map(map)
            }
            Rule::block | Rule::block_labeled => {
                let mut map = Map::new();
                merge_nodes(&mut map, pair.into_inner());
                Node::Map(map)
            }
            Rule::config_file | Rule::block_body_inner => {
                let mut map = Map::new();
                for pair in pair.into_inner() {
                    merge_nodes(&mut map, pair.into_inner());
                }
                Node::Map(map)
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
            Node::Seq(seq) => seq.first().and_then(|n| n.as_pair()),
            Node::Map(map) => map.first().and_then(|(_, n)| n.as_pair()),
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

    fn deep_merge(&mut self, other: &mut Node<'a>) {
        match (self, other) {
            (Node::Map(lhs), Node::Map(rhs)) => {
                rhs.iter_mut().for_each(|(key, value)| {
                    lhs.entry(key.to_string())
                        .and_modify(|lhs| lhs.deep_merge(value))
                        .or_insert_with(|| std::mem::replace(value, Node::Empty));
                });
            }
            (Node::Seq(lhs), Node::Seq(rhs)) => {
                lhs.append(rhs);
            }
            (_, Node::Empty) => (),
            (lhs, rhs) => *lhs = std::mem::replace(rhs, Node::Empty),
        }
    }
}

pub fn interpolate(s: &str) -> String {
    if s.starts_with("${") {
        s.to_owned()
    } else {
        format!("${{{}}}", s)
    }
}

// We need to account for blocks with the same name and merge their contents.
//
// See: https://github.com/hashicorp/hcl/blob/main/json/spec.md#blocks
fn merge_nodes<'a>(map: &mut Map<String, Node<'a>>, pairs: Pairs<'a, Rule>) {
    for (key, mut node) in MapNodesIter::new(pairs) {
        match map.entry(key) {
            Entry::Occupied(mut e) => {
                e.get_mut().deep_merge(&mut node);
            }
            Entry::Vacant(e) => {
                e.insert(node);
            }
        }
    }
}

fn overwrite_nodes<'a>(map: &mut Map<String, Node<'a>>, pairs: Pairs<'a, Rule>) {
    for (key, node) in MapNodesIter::new(pairs) {
        map.insert(key, node);
    }
}

struct MapNodesIter<'a> {
    inner: Pairs<'a, Rule>,
}

impl<'a> MapNodesIter<'a> {
    fn new(inner: Pairs<'a, Rule>) -> Self {
        MapNodesIter { inner }
    }
}

impl<'a> Iterator for MapNodesIter<'a> {
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
