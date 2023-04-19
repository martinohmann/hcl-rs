#![allow(missing_docs)]

use crate::expr::Expression;
use crate::repr::{Decor, Decorate, SetSpan, Span};
use std::ops::Range;

#[derive(Debug, Clone, Eq)]
pub struct Conditional {
    pub cond_expr: Expression,
    pub true_expr: Expression,
    pub false_expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl Conditional {
    pub fn new(
        cond_expr: Expression,
        true_expr: Expression,
        false_expr: Expression,
    ) -> Conditional {
        Conditional {
            cond_expr,
            true_expr,
            false_expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.cond_expr.despan(input);
        self.true_expr.despan(input);
        self.false_expr.despan(input);
    }
}

impl PartialEq for Conditional {
    fn eq(&self, other: &Self) -> bool {
        self.cond_expr == other.cond_expr
            && self.true_expr == other.true_expr
            && self.false_expr == other.false_expr
    }
}

decorate_impl!(Conditional);
span_impl!(Conditional);
