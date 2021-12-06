use super::Rule;
use indexmap::{map::Entry, IndexMap as Map};
use pest::iterators::Pair;

#[derive(Debug, PartialEq, Clone)]
pub enum Node<'a> {
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
            Rule::boolean => Node::Boolean(pair),
            Rule::float => Node::Float(pair),
            Rule::heredoc => Node::String(pair.into_inner().nth(1).unwrap()),
            Rule::identifier => Node::String(pair),
            Rule::int => Node::Int(pair),
            Rule::null => Node::Null(pair),
            Rule::string_lit => Node::String(pair.into_inner().next().unwrap()),
            Rule::tuple | Rule::block_body => {
                Node::Seq(pair.into_inner().map(Node::from_pair).collect())
            }
            Rule::attribute | Rule::block | Rule::block_labeled | Rule::object => {
                let mut map = Map::new();
                let mut inner = pair.into_inner();

                while inner.peek().is_some() {
                    match (inner.next(), inner.next()) {
                        (Some(k), Some(v)) => {
                            map.insert(Node::from_pair(k).as_map_key(), Node::from_pair(v));
                        }
                        (Some(k), None) => panic!("missing map value for key: {}", k),
                        (_, _) => (),
                    };
                }

                Node::Map(map)
            }
            Rule::config_file | Rule::block_body_inner => {
                let mut map = Map::new();

                pair.into_inner()
                    .map(|pair| pair.into_inner())
                    .for_each(|mut inner| {
                        while inner.peek().is_some() {
                            match (inner.next(), inner.next()) {
                                (Some(k), Some(v)) => {
                                    let key = Node::from_pair(k).as_map_key();
                                    let mut value = Node::from_pair(v);

                                    // We need to account for blocks with the same name and merge
                                    // their contents.
                                    //
                                    // See: https://github.com/hashicorp/hcl/blob/main/json/spec.md#blocks
                                    match map.entry(key) {
                                        Entry::Occupied(mut e) => {
                                            match (&mut e.get_mut(), &mut value) {
                                                (Node::Seq(lhs), Node::Seq(rhs)) => lhs.append(rhs),
                                                (_, _) => {
                                                    e.insert(value);
                                                }
                                            }
                                        }
                                        Entry::Vacant(e) => {
                                            e.insert(value);
                                        }
                                    }
                                }
                                (Some(k), None) => panic!("missing map value for key: {}", k),
                                (_, _) => (),
                            };
                        }
                    });

                Node::Map(map)
            }
            _ => Node::Expression(pair),
        }
    }

    fn as_map_key(&self) -> String {
        let s = match self {
            Node::Null(pair) => pair.as_str(),
            Node::Boolean(pair) => pair.as_str(),
            Node::Int(pair) => pair.as_str(),
            Node::Float(pair) => pair.as_str(),
            Node::String(pair) => pair.as_str(),
            Node::Expression(pair) => return interpolate(pair.as_str()),
            node => panic!("unexpected map key: {:?}", node),
        };

        s.to_owned()
    }
}

pub fn interpolate(s: &str) -> String {
    if s.starts_with("${") {
        s.to_owned()
    } else {
        format!("${{{}}}", s)
    }
}
