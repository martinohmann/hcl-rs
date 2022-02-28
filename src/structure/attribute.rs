//! Types to represent and build HCL attributes.

use crate::Value;
use std::iter;

/// Represents an HCL attribute which consists of an attribute key and value.
///
/// In HCL syntax this is represented as:
///
/// ```hcl
/// key = value
/// ```
///
/// Use [`Attribute::new`] to construct an [`Attribute`] from a value that is convertible to this
/// crate's [`Value`] type.
#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    /// The HCL attribute's key.
    pub key: String,
    /// The value of the HCL attribute.
    pub value: Value,
}

impl Attribute {
    /// Creates a new `Attribute` from an attribute key that is convertible into a `String` and an
    /// attribute value that is convertible into a `Value`.
    pub fn new<K, V>(key: K, value: V) -> Attribute
    where
        K: Into<String>,
        V: Into<Value>,
    {
        Attribute {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Returns a reference to the attribute key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns a reference to the attribute value.
    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl From<Attribute> for Value {
    fn from(attr: Attribute) -> Value {
        Value::from_iter(iter::once((attr.key, attr.value)))
    }
}

impl<K, V> From<(K, V)> for Attribute
where
    K: Into<String>,
    V: Into<Value>,
{
    fn from(pair: (K, V)) -> Attribute {
        Attribute::new(pair.0.into(), pair.1.into())
    }
}
