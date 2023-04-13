#![allow(missing_docs)]

use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::Ident;
use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, Eq)]
pub struct Traversal {
    pub expr: Expression,
    pub operators: Vec<Decorated<TraversalOperator>>,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl Traversal {
    pub fn new(expr: Expression, operators: Vec<Decorated<TraversalOperator>>) -> Traversal {
        Traversal {
            expr,
            operators,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);

        for operator in &mut self.operators {
            operator.despan(input);
        }
    }
}

impl PartialEq for Traversal {
    fn eq(&self, other: &Self) -> bool {
        self.expr == other.expr && self.operators == other.operators
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalOperator {
    AttrSplat(Decorated<Splat>),
    FullSplat(Decorated<Splat>),
    GetAttr(Decorated<Ident>),
    Index(Expression),
    LegacyIndex(Decorated<u64>),
}

impl TraversalOperator {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            TraversalOperator::AttrSplat(splat) | TraversalOperator::FullSplat(splat) => {
                splat.decor_mut().despan(input);
            }
            TraversalOperator::GetAttr(ident) => ident.decor_mut().despan(input),
            TraversalOperator::Index(expr) => expr.despan(input),
            TraversalOperator::LegacyIndex(index) => index.decor_mut().despan(input),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Splat;

impl fmt::Display for Splat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "*")
    }
}

decorate_impl!(Traversal);
span_impl!(Traversal);
forward_decorate_impl!(TraversalOperator => { AttrSplat, FullSplat, GetAttr, Index, LegacyIndex });
forward_span_impl!(TraversalOperator => { AttrSplat, FullSplat, GetAttr, Index, LegacyIndex });
