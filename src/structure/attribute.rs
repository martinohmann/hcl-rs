//! Types to represent and build HCL attributes.

use super::{Expression, Value};
use serde::Deserialize;
use std::iter;

/// Represents an HCL attribute which consists of an attribute key and a value expression.
///
/// In HCL syntax this is represented as:
///
/// ```hcl
/// key = value
/// ```
///
/// Use [`Attribute::new`] to construct an [`Attribute`] from a value that is convertible to this
/// crate's [`Expression`] type.
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Attribute {
    /// The HCL attribute's key.
    pub key: String,
    /// The value expression of the HCL attribute.
    pub expr: Expression,
}

impl Attribute {
    /// Creates a new `Attribute` from an attribute key that is convertible into a `String` and an
    /// attribute value that is convertible into an `Expression`.
    pub fn new<K, V>(key: K, expr: V) -> Attribute
    where
        K: Into<String>,
        V: Into<Expression>,
    {
        Attribute {
            key: key.into(),
            expr: expr.into(),
        }
    }

    /// Returns a reference to the attribute key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns a reference to the attribute value expression.
    pub fn expr(&self) -> &Expression {
        &self.expr
    }
}

impl From<Attribute> for Value {
    fn from(attr: Attribute) -> Value {
        Value::from_iter(iter::once((attr.key, attr.expr)))
    }
}

impl<K, V> From<(K, V)> for Attribute
where
    K: Into<String>,
    V: Into<Expression>,
{
    fn from(pair: (K, V)) -> Attribute {
        Attribute::new(pair.0.into(), pair.1.into())
    }
}
