use crate::encode::{Encode, EncodeState};
use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::{Ident, InternalString, RawString};
use std::fmt;
use std::ops::Range;

pub type Iter<'a> = Box<dyn Iterator<Item = &'a Structure> + 'a>;

pub type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Structure> + 'a>;

pub type BlockLabelIter<'a> = Box<dyn Iterator<Item = &'a BlockLabel> + 'a>;

pub type BlockLabelIterMut<'a> = Box<dyn Iterator<Item = &'a mut BlockLabel> + 'a>;

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

    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.structures.iter())
    }

    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.structures.iter_mut())
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        for structure in &mut self.structures {
            structure.despan(input);
        }
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode(&mut state)
    }
}

#[derive(Debug, Clone)]
pub enum Structure {
    Attribute(Attribute),
    Block(Block),
}

forward_decorate_span_impl!(Structure => { Attribute, Block });

impl Structure {
    pub(crate) fn despan(&mut self, input: &str) {
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

    pub(crate) fn despan(&mut self, input: &str) {
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
        Block {
            identifier: ident,
            labels: Vec::new(),
            body,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn ident(&self) -> &Decorated<Ident> {
        &self.identifier
    }

    pub fn labels(&self) -> BlockLabelIter<'_> {
        Box::new(self.labels.iter())
    }

    pub fn labels_mut(&mut self) -> BlockLabelIterMut<'_> {
        Box::new(self.labels.iter_mut())
    }

    pub fn set_labels(&mut self, labels: Vec<BlockLabel>) {
        self.labels = labels;
    }

    pub fn body(&self) -> &BlockBody {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut BlockBody {
        &mut self.body
    }

    pub(crate) fn despan(&mut self, input: &str) {
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
    Ident(Decorated<Ident>),
    String(Decorated<InternalString>),
}

forward_decorate_span_impl!(BlockLabel => { Ident, String });

impl BlockLabel {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            BlockLabel::Ident(ident) => ident.decor_mut().despan(input),
            BlockLabel::String(expr) => expr.decor_mut().despan(input),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockBody {
    Multiline(Body),
    Oneline(Attribute),
    Empty(RawString),
}

impl BlockBody {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            BlockBody::Multiline(body) => body.despan(input),
            BlockBody::Oneline(attr) => attr.despan(input),
            BlockBody::Empty(raw) => raw.despan(input),
        }
    }
}
