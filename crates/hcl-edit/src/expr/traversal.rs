use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated};
use crate::Ident;
use std::fmt;
use std::ops::Range;

/// Traverse an expression to access attributes, object keys or element indices.
#[derive(Debug, Clone, Eq)]
pub struct Traversal {
    /// The expression that the access operator is applied to.
    pub expr: Expression,
    /// The traversal operators to apply to `expr` one of the other.
    pub operators: Vec<Decorated<TraversalOperator>>,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl Traversal {
    /// Creates a new `Traversal` structure from an expression and traversal operators that should
    /// be applied to it.
    pub fn new(
        expr: impl Into<Expression>,
        operators: Vec<Decorated<TraversalOperator>>,
    ) -> Traversal {
        Traversal {
            expr: expr.into(),
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

/// The expression traversal operators that are supported by HCL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraversalOperator {
    /// The attribute-only splat operator supports only attribute lookups into the elements from a
    /// list, but supports an arbitrary number of them.
    AttrSplat(Decorated<Splat>),
    /// The full splat operator additionally supports indexing into the elements from a list, and
    /// allows any combination of attribute access and index operations.
    FullSplat(Decorated<Splat>),
    /// The attribute access operator returns the value of a single attribute in an object value.
    GetAttr(Decorated<Ident>),
    /// The index operator returns the value of a single element of a collection value based on
    /// the result of the expression.
    Index(Expression),
    /// The legacy index operator returns the value of a single element of a collection value.
    /// Exists only for compatibility with the precursor language HIL. Use the `Index` variant
    /// instead.
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

/// Represents the splat operator (`*`) that is used within a
/// [`AttrSplat`](TraversalOperator::AttrSplat) or [`FullSplat`](TraversalOperator::FullSplat).
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
