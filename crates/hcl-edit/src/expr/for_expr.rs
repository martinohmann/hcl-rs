use crate::expr::Expression;
use crate::{Decor, Decorate, Decorated, Ident};
use std::ops::Range;

/// A for expression is a construct for constructing a collection by projecting the items from
/// another collection.
#[derive(Debug, Clone, Eq)]
pub struct ForExpr {
    /// The `for` expression introduction, containing an optional key var, value var and the
    /// collection expression that is iterated.
    pub intro: ForIntro,
    /// An expression that is evaluated once for each key in the source collection. If set, the
    /// result of the `for` expression will be an object. Otherwise, the result will be an array.
    pub key_expr: Option<Expression>,
    /// An expression that is evaluated once for each value in the source collection.
    pub value_expr: Expression,
    /// Indicates whether grouping mode is enabled. In grouping mode, each value in the resulting
    /// object is a list of all of the values that were produced against each distinct key. This is
    /// ignored if `key_expr` is `None`.
    pub grouping: bool,
    /// An optional filter expression. Elements for which the condition evaluates to `true` will
    /// be evaluated as normal, while if `false` the element will be skipped.
    pub cond: Option<ForCond>,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl ForExpr {
    /// Creates a new `ForExpr` from a `for` expression introduction and a result value
    /// expression.
    pub fn new(intro: ForIntro, value_expr: impl Into<Expression>) -> ForExpr {
        ForExpr {
            intro,
            key_expr: None,
            value_expr: value_expr.into(),
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

/// The `for` expression introduction, containing an optional key var, value var and the
/// collection expression that is iterated.
#[derive(Debug, Clone, Eq)]
pub struct ForIntro {
    /// Optional name of the variable that will be temporarily assigned the key of each element
    /// during iteration. If the source collection is an array, it gets assigned the zero-based
    /// array index. For an object source collection, this gets assigned the object's key.
    pub key_var: Option<Decorated<Ident>>,
    /// The name of the variable that will be temporarily assigned the value of each element
    /// during iteration.
    pub value_var: Decorated<Ident>,
    /// An expression that must evaluate to a value that can be iterated.
    pub collection_expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl ForIntro {
    /// Creates a new `ForIntro` from a value variable and a collection expression.
    pub fn new(
        value_var: impl Into<Decorated<Ident>>,
        collection_expr: impl Into<Expression>,
    ) -> ForIntro {
        ForIntro {
            key_var: None,
            value_var: value_var.into(),
            collection_expr: collection_expr.into(),
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

/// A filter expression. Elements for which the condition evaluates to `true` will be evaluated as
/// normal, while if `false` the element will be skipped.
#[derive(Debug, Clone, Eq)]
pub struct ForCond {
    /// The filter expression.
    pub expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl ForCond {
    /// Creates a new `ForCond` from an expression.
    pub fn new(expr: impl Into<Expression>) -> ForCond {
        ForCond {
            expr: expr.into(),
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
