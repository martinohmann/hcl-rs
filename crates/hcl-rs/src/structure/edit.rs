use super::*;
use crate::edit::structure;

impl From<structure::Body> for Body {
    fn from(value: structure::Body) -> Self {
        Body::from_iter(value)
    }
}

impl From<Body> for structure::Body {
    fn from(value: Body) -> Self {
        structure::Body::from_iter(value)
    }
}

impl From<structure::Structure> for Structure {
    fn from(value: structure::Structure) -> Self {
        match value {
            structure::Structure::Attribute(attr) => Structure::Attribute(attr.into()),
            structure::Structure::Block(block) => Structure::Block(block.into()),
        }
    }
}

impl From<Structure> for structure::Structure {
    fn from(value: Structure) -> Self {
        match value {
            Structure::Attribute(attr) => structure::Structure::Attribute(attr.into()),
            Structure::Block(block) => structure::Structure::Block(block.into()),
        }
    }
}

impl From<structure::Attribute> for Attribute {
    fn from(value: structure::Attribute) -> Self {
        Attribute {
            key: value.key.into(),
            expr: value.value.into(),
        }
    }
}

impl From<Attribute> for structure::Attribute {
    fn from(value: Attribute) -> Self {
        structure::Attribute::new(value.key, value.expr)
    }
}

impl From<structure::Block> for Block {
    fn from(value: structure::Block) -> Self {
        Block::builder(value.ident)
            .add_labels(value.labels)
            .add_structures(value.body)
            .build()
    }
}

impl From<Block> for structure::Block {
    fn from(value: Block) -> Self {
        structure::Block::builder(value.identifier)
            .labels(value.labels)
            .structures(value.body)
            .build()
    }
}

impl From<structure::BlockLabel> for BlockLabel {
    fn from(value: structure::BlockLabel) -> Self {
        match value {
            structure::BlockLabel::Ident(ident) => BlockLabel::Identifier(ident.into()),
            structure::BlockLabel::String(string) => BlockLabel::String(string.value_into()),
        }
    }
}

impl From<BlockLabel> for structure::BlockLabel {
    fn from(value: BlockLabel) -> Self {
        match value {
            BlockLabel::Identifier(ident) => structure::BlockLabel::Ident(ident.into()),
            BlockLabel::String(string) => structure::BlockLabel::String(string.into()),
        }
    }
}
