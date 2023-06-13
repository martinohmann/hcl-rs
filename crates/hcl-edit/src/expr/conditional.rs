use crate::expr::Expression;
use crate::repr::Decor;
use std::ops::Range;

/// The conditional operator allows selecting from one of two expressions based on the outcome of a
/// boolean expression.
#[derive(Debug, Clone, Eq)]
pub struct Conditional {
    /// A condition expression that evaluates to a boolean value.
    pub cond_expr: Expression,
    /// The expression returned by the conditional if the condition evaluates to `true`.
    pub true_expr: Expression,
    /// The expression returned by the conditional if the condition evaluates to `false`.
    pub false_expr: Expression,

    decor: Decor,
    span: Option<Range<usize>>,
}

impl Conditional {
    /// Creates a new `Conditional` from a condition and two expressions for the branches of the
    /// conditional.
    pub fn new(
        cond_expr: impl Into<Expression>,
        true_expr: impl Into<Expression>,
        false_expr: impl Into<Expression>,
    ) -> Conditional {
        Conditional {
            cond_expr: cond_expr.into(),
            true_expr: true_expr.into(),
            false_expr: false_expr.into(),
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
