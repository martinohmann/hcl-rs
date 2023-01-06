use super::Expression;
use crate::Identifier;
use serde::{Deserialize, Serialize};

/// Traverse an expression to access attributes, object keys or element indices.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
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

    /// Create a new `TraversalBuilder` for the given expression.
    pub fn builder<T>(expr: T) -> TraversalBuilder
    where
        T: Into<Expression>,
    {
        TraversalBuilder {
            expr: expr.into(),
            operators: Vec::new(),
        }
    }
}

/// A builder for expression traversals.
///
/// It is constructed via the [`builder`][Traversal::builder] method of the [`Traversal`] type.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use hcl::expr::{Traversal, Variable};
///
/// let traversal = Traversal::builder(Variable::new("var")?)
///     .attr("some_array")
///     .index(0)
///     .build();
///
/// // Serializes as `var.some_array[0]`.
/// #     Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct TraversalBuilder {
    expr: Expression,
    operators: Vec<TraversalOperator>,
}

impl TraversalBuilder {
    /// Add an [attribute access operator][TraversalOperator::GetAttr] to the traversal chain.
    pub fn attr<T>(mut self, ident: T) -> Self
    where
        T: Into<Identifier>,
    {
        self.operators
            .push(TraversalOperator::GetAttr(ident.into()));
        self
    }

    /// Add an [attribute splat operator][TraversalOperator::AttrSplat] to the traversal chain.
    pub fn attr_splat(mut self) -> Self {
        self.operators.push(TraversalOperator::AttrSplat);
        self
    }

    /// Add a [full splat operator][TraversalOperator::FullSplat] to the traversal chain.
    pub fn full_splat(mut self) -> Self {
        self.operators.push(TraversalOperator::FullSplat);
        self
    }

    /// Add an [index operator][TraversalOperator::Index] to the traversal chain.
    pub fn index<T>(mut self, expr: T) -> Self
    where
        T: Into<Expression>,
    {
        self.operators.push(TraversalOperator::Index(expr.into()));
        self
    }

    /// Consume `self` and return a `Traversal`.
    pub fn build(self) -> Traversal {
        Traversal {
            expr: self.expr,
            operators: self.operators,
        }
    }
}

/// The expression traversal operators that are supported by HCL.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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
