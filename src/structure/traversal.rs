use crate::{Expression, Identifier};
use serde::{Deserialize, Serialize};

/// Traverse an expression to access attributes, object keys or element indices.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename = "$hcl::traversal")]
pub struct Traversal {
    /// The expression that the access operator is applied to.
    pub expr: Expression,
    /// The traversal operators to apply to `expr` one of the other.
    pub operators: Vec<TraversalOperator>,
}

impl Traversal {
    /// Creates a new `Traversal` structure from an expression and traversal operators that should
    /// be applied to it.
    pub fn new<E, I>(expr: E, operators: I) -> Self
    where
        E: Into<Expression>,
        I: IntoIterator,
        I::Item: Into<TraversalOperator>,
    {
        Traversal {
            expr: expr.into(),
            operators: operators.into_iter().map(Into::into).collect(),
        }
    }
}

/// The expression traversal operators that are supported by HCL.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename = "$hcl::traversal_operator")]
pub enum TraversalOperator {
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

impl<T> From<T> for TraversalOperator
where
    T: Into<Identifier>,
{
    fn from(value: T) -> TraversalOperator {
        TraversalOperator::GetAttr(value.into())
    }
}

impl From<Expression> for TraversalOperator {
    fn from(value: Expression) -> TraversalOperator {
        TraversalOperator::Index(value)
    }
}

impl From<u64> for TraversalOperator {
    fn from(value: u64) -> TraversalOperator {
        TraversalOperator::LegacyIndex(value)
    }
}
