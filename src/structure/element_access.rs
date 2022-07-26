use crate::{Expression, Identifier};
use serde::{Deserialize, Serialize};

/// Access to an element of an expression result.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename = "$hcl::element_access")]
pub struct ElementAccess {
    /// The expression that the access operator is applied to.
    pub expr: Expression,
    /// The element access operators used on the expression.
    pub operators: Vec<ElementAccessOperator>,
}

/// The kinds of element access that are supported by HCL.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename = "$hcl::element_access_operator")]
pub enum ElementAccessOperator {
    /// The attribute-only splat operator supports only attribute lookups into the elements from a
    /// list, but supports an arbitrary number of them.
    AttrSplat,
    /// The full splat operator additionally supports indexing into the elements from a list, and
    /// allows any combination of attribute access and index operations.
    FullSplat,
    /// The attribute access operator returns the value of a single attribute in an object value.
    GetAttr(Identifier),
    /// The index operator returns the value of a single element of a collection value based on
    /// the result of the expression.
    Index(Expression),
    /// The legacy index operator returns the value of a single element of a collection value.
    /// Exists only for compatibility with the precursor language HIL. Use the `Index` variant
    /// instead.
    LegacyIndex(u64),
}

impl ElementAccess {
    /// Creates a new ElementAccess structure.
    pub fn new<E, I>(expr: E, operators: I) -> Self
    where
        E: Into<Expression>,
        I: IntoIterator,
        I::Item: Into<ElementAccessOperator>,
    {
        ElementAccess {
            expr: expr.into(),
            operators: operators.into_iter().map(Into::into).collect(),
        }
    }
}
