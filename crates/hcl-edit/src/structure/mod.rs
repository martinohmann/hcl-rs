//! Types to represent the HCL structural sub-language.

#![allow(missing_docs)]

use crate::encode::{Encode, EncodeState};
use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::{parser, Ident, RawString};
use std::fmt;
use std::ops::Range;
use std::str::FromStr;

pub type Iter<'a> = Box<dyn Iterator<Item = &'a Structure> + 'a>;

pub type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Structure> + 'a>;

pub type LabelIter<'a> = Box<dyn Iterator<Item = &'a BlockLabel> + 'a>;

pub type LabelIterMut<'a> = Box<dyn Iterator<Item = &'a mut BlockLabel> + 'a>;

#[derive(Debug, Clone, Default, Eq)]
pub struct Body {
    structures: Vec<Structure>,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl Body {
    pub fn new() -> Body {
        Body::default()
    }

    pub fn is_empty(&self) -> bool {
        self.structures.is_empty()
    }

    pub fn len(&self) -> usize {
        self.structures.len()
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

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.structures == other.structures
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode(&mut state)
    }
}

impl FromStr for Body {
    type Err = parser::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse_body(s)
    }
}

impl From<Vec<Structure>> for Body {
    fn from(structures: Vec<Structure>) -> Self {
        Body {
            structures,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Structure {
    Attribute(Attribute),
    Block(Block),
}

impl Structure {
    pub fn is_attribute(&self) -> bool {
        self.as_attribute().is_some()
    }

    pub fn is_block(&self) -> bool {
        self.as_block().is_some()
    }

    pub fn as_attribute(&self) -> Option<&Attribute> {
        match self {
            Structure::Attribute(attr) => Some(attr),
            Structure::Block(_) => None,
        }
    }

    pub fn as_attribute_mut(&mut self) -> Option<&mut Attribute> {
        match self {
            Structure::Attribute(attr) => Some(attr),
            Structure::Block(_) => None,
        }
    }

    pub fn as_block(&self) -> Option<&Block> {
        match self {
            Structure::Block(block) => Some(block),
            Structure::Attribute(_) => None,
        }
    }

    pub fn as_block_mut(&mut self) -> Option<&mut Block> {
        match self {
            Structure::Block(block) => Some(block),
            Structure::Attribute(_) => None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            Structure::Attribute(attr) => attr.despan(input),
            Structure::Block(block) => block.despan(input),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Attribute {
    key: Decorated<Ident>,
    expr: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

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

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.expr == other.expr
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Block {
    identifier: Decorated<Ident>,
    labels: Vec<BlockLabel>,
    body: BlockBody,
    decor: Decor,
    span: Option<Range<usize>>,
}

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

    pub fn labels(&self) -> LabelIter<'_> {
        Box::new(self.labels.iter())
    }

    pub fn labels_mut(&mut self) -> LabelIterMut<'_> {
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

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
            && self.labels == other.labels
            && self.body == other.body
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockLabel {
    Ident(Decorated<Ident>),
    String(Decorated<String>),
}

impl BlockLabel {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            BlockLabel::Ident(ident) => ident.decor_mut().despan(input),
            BlockLabel::String(expr) => expr.decor_mut().despan(input),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockBody {
    Multiline(Body),
    Oneline(Box<Oneline>),
}

impl BlockBody {
    pub fn is_empty(&self) -> bool {
        match self {
            BlockBody::Multiline(body) => body.is_empty(),
            BlockBody::Oneline(oneline) => oneline.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            BlockBody::Multiline(body) => body.len(),
            BlockBody::Oneline(oneline) => {
                if oneline.is_empty() {
                    0
                } else {
                    1
                }
            }
        }
    }

    pub fn is_multiline(&self) -> bool {
        self.as_multiline().is_some()
    }

    pub fn is_oneline(&self) -> bool {
        self.as_oneline().is_some()
    }

    pub fn as_multiline(&self) -> Option<&Body> {
        match self {
            BlockBody::Multiline(body) => Some(body),
            BlockBody::Oneline(_) => None,
        }
    }

    pub fn as_multiline_mut(&mut self) -> Option<&mut Body> {
        match self {
            BlockBody::Multiline(body) => Some(body),
            BlockBody::Oneline(_) => None,
        }
    }

    pub fn as_oneline(&self) -> Option<&Oneline> {
        match self {
            BlockBody::Multiline(_) => None,
            BlockBody::Oneline(oneline) => Some(oneline),
        }
    }

    pub fn as_oneline_mut(&mut self) -> Option<&mut Oneline> {
        match self {
            BlockBody::Multiline(_) => None,
            BlockBody::Oneline(oneline) => Some(oneline),
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        match self {
            BlockBody::Multiline(body) => body.iter(),
            BlockBody::Oneline(oneline) => match &oneline.attr {
                Some(attr) => Box::new(std::iter::once(attr)),
                None => Box::new(std::iter::empty()),
            },
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_> {
        match self {
            BlockBody::Multiline(body) => body.iter_mut(),
            BlockBody::Oneline(oneline) => match &mut oneline.attr {
                Some(attr) => Box::new(std::iter::once(attr)),
                None => Box::new(std::iter::empty()),
            },
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            BlockBody::Multiline(body) => body.despan(input),
            BlockBody::Oneline(oneline) => oneline.despan(input),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Oneline {
    attr: Option<Structure>,
    trailing: RawString,
}

impl Oneline {
    pub fn new() -> Oneline {
        Oneline::default()
    }

    pub fn is_empty(&self) -> bool {
        self.attr.is_none()
    }

    pub fn set_attribute(&mut self, attr: Attribute) {
        self.attr = Some(Structure::Attribute(attr))
    }

    pub fn as_attribute(&self) -> Option<&Attribute> {
        self.attr.as_ref().and_then(|s| s.as_attribute())
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        if let Some(attr) = &mut self.attr {
            attr.despan(input);
        }
        self.trailing.despan(input);
    }
}

impl From<Attribute> for Oneline {
    fn from(attr: Attribute) -> Self {
        Oneline {
            attr: Some(Structure::Attribute(attr)),
            trailing: RawString::default(),
        }
    }
}

decorate_impl! { Body, Attribute, Block }

span_impl! { Body, Attribute, Block }

forward_decorate_impl! {
    Structure => { Attribute, Block },
    BlockLabel => { Ident, String },
}

forward_span_impl! {
    Structure => { Attribute, Block },
    BlockLabel => { Ident, String }
}
