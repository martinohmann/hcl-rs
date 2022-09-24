use crate::{Expression, Identifier};
use serde::{Deserialize, Serialize};

/// Traversal an expression to access object attributes or element indices.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename = "$hcl::traversal")]
pub struct Traversal {
    /// The expression that the access operator is applied to.
    pub expr: Expression,
    /// The element access operator used on the expression.
    pub operator: TraversalOperator,
}

impl Traversal {
    /// Creates a new `Traversal` structure from a traversal operator and and expression that it
    /// should be applied to.
    pub fn new<E, O>(expr: E, operator: O) -> Self
    where
        E: Into<Expression>,
        O: Into<TraversalOperator>,
    {
        Traversal {
            expr: expr.into(),
            operator: operator.into(),
        }
    }

    /// Chains another `TraversalOperator` and returns a new `Traversal`.
    pub fn chain<O>(self, operator: O) -> Traversal
    where
        O: Into<TraversalOperator>,
    {
        Traversal {
            expr: Expression::Traversal(Box::new(self)),
            operator: operator.into(),
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
