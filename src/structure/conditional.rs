use super::Expression;
use serde::{Deserialize, Serialize};

/// The conditional operator allows selecting from one of two expressions based on the outcome of a
/// boolean expression.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename = "$hcl::conditional")]
pub struct Conditional {
    /// A predicate expression that evaluates to a boolean value.
    pub predicate: Expression,
    /// The expression returned by the conditional if the predicate evaluates to `true`.
    pub true_expr: Expression,
    /// The expression returned by the conditional if the predicate evaluates to `false`.
    pub false_expr: Expression,
}

impl Conditional {
    /// Creates a new `Conditional` from a predicate and two expressions for the branches of the
    /// conditional.
    pub fn new<P, T, F>(predicate: P, true_expr: T, false_expr: F) -> Conditional
    where
        P: Into<Expression>,
        T: Into<Expression>,
        F: Into<Expression>,
    {
        Conditional {
            predicate: predicate.into(),
            true_expr: true_expr.into(),
            false_expr: false_expr.into(),
        }
    }
}
