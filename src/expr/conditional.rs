use super::Expression;
use serde::Deserialize;

/// The conditional operator allows selecting from one of two expressions based on the outcome of a
/// boolean expression.
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Conditional {
    /// A condition expression that evaluates to a boolean value.
    pub cond_expr: Expression,
    /// The expression returned by the conditional if the condition evaluates to `true`.
    pub true_expr: Expression,
    /// The expression returned by the conditional if the condition evaluates to `false`.
    pub false_expr: Expression,
}

impl Conditional {
    /// Creates a new `Conditional` from a condition and two expressions for the branches of the
    /// conditional.
    pub fn new<C, T, F>(cond_expr: C, true_expr: T, false_expr: F) -> Conditional
    where
        C: Into<Expression>,
        T: Into<Expression>,
        F: Into<Expression>,
    {
        Conditional {
            cond_expr: cond_expr.into(),
            true_expr: true_expr.into(),
            false_expr: false_expr.into(),
        }
    }
}
