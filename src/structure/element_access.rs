use std::collections::VecDeque;

use crate::{Expression, Identifier};
use serde::{Deserialize, Serialize};

/// Access to an element of an expression result.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename = "$hcl::element_access")]
pub struct ElementAccess {
    /// The expression that the access operator is applied to.
    pub expr: Expression,
    /// The element access operator used on the expression.
    pub operator: ElementAccessOperator,
}

impl ElementAccess {
    /// Creates a new `ElementAccess` structure from an access operator and and expression that it
    /// should be applied to.
    pub fn new<E, O>(expr: E, operator: O) -> Self
    where
        E: Into<Expression>,
        O: Into<ElementAccessOperator>,
    {
        ElementAccess {
            expr: expr.into(),
            operator: operator.into(),
        }
    }

    /// Chains another `ElementAccessOperator` and returns a new `ElementAccess`.
    pub fn chain<O>(self, operator: O) -> ElementAccess
    where
        O: Into<ElementAccessOperator>,
    {
        ElementAccess {
            expr: Expression::ElementAccess(Box::new(self)),
            operator: operator.into(),
        }
    }

    pub(crate) fn flatten(self) -> (Expression, VecDeque<ElementAccessOperator>) {
        let mut operators = VecDeque::with_capacity(1);
        let mut expr = Expression::from(self);

        while let Expression::ElementAccess(access) = expr {
            operators.push_front(access.operator);
            expr = access.expr
        }

        (expr, operators)
    }
}

/// The kinds of element access that are supported by HCL.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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

impl<T> From<T> for ElementAccessOperator
where
    T: Into<Identifier>,
{
    fn from(value: T) -> ElementAccessOperator {
        ElementAccessOperator::GetAttr(value.into())
    }
}

impl From<Expression> for ElementAccessOperator {
    fn from(value: Expression) -> ElementAccessOperator {
        ElementAccessOperator::Index(value)
    }
}

impl From<u64> for ElementAccessOperator {
    fn from(value: u64) -> ElementAccessOperator {
        ElementAccessOperator::LegacyIndex(value)
    }
}
