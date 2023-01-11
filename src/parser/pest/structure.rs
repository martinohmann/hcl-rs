use super::*;
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};

pub fn body(pair: Pair<Rule>) -> Result<Body> {
    pair.into_inner().map(structure).collect()
}

fn structure(pair: Pair<Rule>) -> Result<Structure> {
    match pair.as_rule() {
        Rule::Attribute => attribute(pair).map(Structure::Attribute),
        Rule::Block => block(pair).map(Structure::Block),
        rule => unexpected_rule(rule),
    }
}

fn attribute(pair: Pair<Rule>) -> Result<Attribute> {
    let mut pairs = pair.into_inner();

    Ok(Attribute {
        key: ident(pairs.next().unwrap()),
        expr: expression(pairs.next().unwrap())?,
    })
}

fn block(pair: Pair<Rule>) -> Result<Block> {
    let mut pairs = pair.into_inner();
    let identifier = ident(pairs.next().unwrap());
    let (labels, body): (Vec<Pair<Rule>>, Vec<Pair<Rule>>) =
        pairs.partition(|pair| pair.as_rule() != Rule::BlockBody);

    Ok(Block {
        identifier,
        labels: labels.into_iter().map(block_label).collect::<Result<_>>()?,
        body: block_body(body.into_iter().next().unwrap())?,
    })
}

fn block_label(pair: Pair<Rule>) -> Result<BlockLabel> {
    match pair.as_rule() {
        Rule::Identifier => Ok(BlockLabel::Identifier(ident(pair))),
        Rule::StringLit => unescape_string(inner(pair)).map(BlockLabel::String),
        rule => unexpected_rule(rule),
    }
}

fn block_body(pair: Pair<Rule>) -> Result<Body> {
    match pair.as_rule() {
        Rule::BlockBody => body(inner(pair)),
        rule => unexpected_rule(rule),
    }
}
