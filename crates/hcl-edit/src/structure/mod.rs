use crate::encode::{Encode, EncodeState};
use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, Despan, RawString, SetSpan, Span};
use hcl_primitives::{Ident, InternalString};
use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, Default)]
pub struct Body {
    structures: Vec<Structure>,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Body);

impl Body {
    pub fn new(structures: Vec<Structure>) -> Body {
        Body {
            structures,
            ..Default::default()
        }
    }

    pub fn structures(&self) -> &[Structure] {
        &self.structures
    }
}

impl Despan for Body {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        for structure in &mut self.structures {
            structure.despan(input);
        }
    }
}

impl From<Vec<Structure>> for Body {
    fn from(structures: Vec<Structure>) -> Self {
        Body::new(structures)
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f, None);
        self.encode(&mut state)
    }
}

#[derive(Debug, Clone)]
pub enum Structure {
    Attribute(Box<Attribute>),
    Block(Box<Block>),
}

forward_decorate_span_impl!(Structure => { Attribute, Block });

impl Despan for Structure {
    fn despan(&mut self, input: &str) {
        match self {
            Structure::Attribute(attr) => attr.despan(input),
            Structure::Block(block) => block.despan(input),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    key: Decorated<Ident>,
    expr: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Attribute);

impl Attribute {
    pub fn new(key: Decorated<Ident>, expr: Expression) -> Attribute {
        Attribute {
            key,
            expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn key(&self) -> &Decorated<Ident> {
        &self.key
    }

    pub fn expr(&self) -> &Expression {
        &self.expr
    }
}

impl Despan for Attribute {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.key.decor_mut().despan(input);
        self.expr.despan(input);
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    identifier: Decorated<Ident>,
    labels: Vec<BlockLabel>,
    body: BlockBody,
    decor: Decor,
    span: Option<Range<usize>>,
}

decorate_span_impl!(Block);

impl Block {
    pub fn new(ident: Decorated<Ident>, body: BlockBody) -> Block {
        Block::new_with_labels(ident, Vec::new(), body)
    }

    pub fn new_with_labels(
        ident: Decorated<Ident>,
        labels: Vec<BlockLabel>,
        body: BlockBody,
    ) -> Block {
        Block {
            identifier: ident,
            labels,
            body,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn ident(&self) -> &Decorated<Ident> {
        &self.identifier
    }

    pub fn labels(&self) -> &[BlockLabel] {
        &self.labels
    }

    pub fn body(&self) -> &BlockBody {
        &self.body
    }
}

impl Despan for Block {
    fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.identifier.decor_mut().despan(input);
        for label in &mut self.labels {
            label.despan(input);
        }
        self.body.despan(input);
    }
}

#[derive(Debug, Clone)]
pub enum BlockLabel {
    Identifier(Decorated<Ident>),
    String(Decorated<InternalString>),
}

forward_decorate_span_impl!(BlockLabel => { Identifier, String });

impl Despan for BlockLabel {
    fn despan(&mut self, input: &str) {
        match self {
            BlockLabel::Identifier(ident) => ident.decor_mut().despan(input),
            BlockLabel::String(expr) => expr.decor_mut().despan(input),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockBody {
    Multiline(Box<Body>),
    Oneline(Box<Attribute>),
    Empty(RawString),
}

impl Despan for BlockBody {
    fn despan(&mut self, input: &str) {
        match self {
            BlockBody::Multiline(body) => body.despan(input),
            BlockBody::Oneline(attr) => attr.despan(input),
            BlockBody::Empty(raw) => raw.despan(input),
        }
    }
}
