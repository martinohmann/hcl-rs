use crate::Value;
use std::iter;

#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: Value,
}

impl Attribute {
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
