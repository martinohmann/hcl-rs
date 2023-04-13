use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span};
use crate::Ident;
use std::ops::Range;

#[derive(Debug, Clone, Eq)]
pub struct ForExpr {
    pub intro: ForIntro,
    pub key_expr: Option<Expression>,
    pub value_expr: Expression,
    pub grouping: bool,
    pub cond: Option<ForCond>,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl ForExpr {
    pub fn new(intro: ForIntro, value_expr: Expression) -> ForExpr {
        ForExpr {
            intro,
            key_expr: None,
            value_expr,
            grouping: false,
            cond: None,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.intro.despan(input);

        if let Some(key_expr) = &mut self.key_expr {
            key_expr.despan(input);
        }

        self.value_expr.despan(input);

        if let Some(cond) = &mut self.cond {
            cond.despan(input);
        }
    }
}

impl PartialEq for ForExpr {
    fn eq(&self, other: &Self) -> bool {
        self.intro == other.intro
            && self.key_expr == other.key_expr
            && self.value_expr == other.value_expr
            && self.grouping == other.grouping
            && self.cond == other.cond
    }
}

#[derive(Debug, Clone, Eq)]
pub struct ForIntro {
    pub key_var: Option<Decorated<Ident>>,
    pub value_var: Decorated<Ident>,
    pub collection_expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl ForIntro {
    pub fn new(value_var: Decorated<Ident>, collection_expr: Expression) -> ForIntro {
        ForIntro {
            key_var: None,
            value_var,
            collection_expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        if let Some(key_var) = &mut self.key_var {
            key_var.decor_mut().despan(input);
        }

        self.value_var.decor_mut().despan(input);
        self.collection_expr.despan(input);
    }
}

impl PartialEq for ForIntro {
    fn eq(&self, other: &Self) -> bool {
        self.key_var == other.key_var
            && self.value_var == other.value_var
            && self.collection_expr == other.collection_expr
    }
}

#[derive(Debug, Clone, Eq)]
pub struct ForCond {
    pub expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl ForCond {
    pub fn new(expr: Expression) -> ForCond {
        ForCond {
            expr,
            decor: Decor::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.expr.despan(input);
    }
}

impl PartialEq for ForCond {
    fn eq(&self, other: &Self) -> bool {
        self.expr == other.expr
    }
}

impl From<Expression> for ForCond {
    fn from(value: Expression) -> Self {
        ForCond::new(value)
    }
}

decorate_impl!(ForExpr, ForIntro, ForCond);
span_impl!(ForExpr, ForIntro, ForCond);
