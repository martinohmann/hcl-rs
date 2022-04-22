//! Types to represent HCL attribute value expressions.

use crate::{Number, Value};
use std::borrow::Cow;
use std::fmt::{self, Display, Write};

/// The object type used in the expression sub-language.
pub type Object<K, V> = indexmap::IndexMap<K, V>;

/// A type representing the expression sub-language is used within attribute definitions to specify
/// values.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Expression {
    /// Represents a null value.
    Null,
    /// Represents a boolean.
    Bool(bool),
    /// Represents a number, either integer or float.
    Number(Number),
    /// Represents a string.
    String(String),
    /// Represents array.
    Array(Vec<Expression>),
    /// Represents an object.
    Object(Object<ObjectKey, Expression>),
    /// Represents a raw HCL expression. This includes any expression kind that does match any of
    /// the enum variants above. See [`RawExpression`] for more details.
    Raw(RawExpression),
}

impl From<Expression> for Value {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::Null => Value::Null,
            Expression::Bool(b) => Value::Bool(b),
            Expression::Number(n) => Value::Number(n),
            Expression::String(s) => Value::String(s),
            Expression::Array(array) => array.into_iter().collect(),
            Expression::Object(object) => object.into_iter().collect(),
            Expression::Raw(raw) => Value::String(raw.into()),
        }
    }
}

impl From<Value> for Expression {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Expression::Null,
            Value::Bool(b) => Expression::Bool(b),
            Value::Number(n) => Expression::Number(n),
            Value::String(s) => Expression::String(s),
            Value::Array(array) => array.into_iter().collect(),
            Value::Object(object) => object.into_iter().collect(),
        }
    }
}

macro_rules! impl_from_integer {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Expression {
                fn from(n: $ty) -> Self {
                    Expression::Number(n.into())
                }
            }
        )*
    };
}

impl_from_integer!(i8, i16, i32, i64, isize);
impl_from_integer!(u8, u16, u32, u64, usize);

impl From<f32> for Expression {
    fn from(f: f32) -> Self {
        Expression::Number(f.into())
    }
}

impl From<f64> for Expression {
    fn from(f: f64) -> Self {
        Expression::Number(f.into())
    }
}

impl From<bool> for Expression {
    fn from(b: bool) -> Self {
        Expression::Bool(b)
    }
}

impl From<String> for Expression {
    fn from(s: String) -> Self {
        Expression::String(s)
    }
}

impl From<&str> for Expression {
    fn from(s: &str) -> Self {
        Expression::String(s.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Expression {
    fn from(s: Cow<'a, str>) -> Self {
        Expression::String(s.into_owned())
    }
}

impl From<Object<ObjectKey, Expression>> for Expression {
    fn from(f: Object<ObjectKey, Expression>) -> Self {
        Expression::Object(f)
    }
}

impl<T: Into<Expression>> From<Vec<T>> for Expression {
    fn from(f: Vec<T>) -> Self {
        Expression::Array(f.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<Expression>> From<&'a [T]> for Expression {
    fn from(f: &'a [T]) -> Self {
        Expression::Array(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<Expression>> FromIterator<T> for Expression {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Expression::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<ObjectKey>, V: Into<Expression>> FromIterator<(K, V)> for Expression {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Expression::Object(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl From<()> for Expression {
    fn from((): ()) -> Self {
        Expression::Null
    }
}

impl From<RawExpression> for Expression {
    fn from(raw: RawExpression) -> Self {
        Expression::Raw(raw)
    }
}

/// Represents an object key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ObjectKey {
    /// Represents a bare unquoted identifer used as object key.
    Identifier(String),
    /// Represents a quoted string used as object key.
    String(String),
    /// Represents a raw HCL expression. This includes any expression kind that does match any of
    /// the enum variants above. See [`RawExpression`] for more details.
    RawExpression(RawExpression),
}

impl From<&str> for ObjectKey {
    fn from(key: &str) -> Self {
        ObjectKey::String(key.into())
    }
}

impl From<String> for ObjectKey {
    fn from(key: String) -> Self {
        ObjectKey::String(key)
    }
}

impl<'a> From<Cow<'a, str>> for ObjectKey {
    fn from(key: Cow<'a, str>) -> Self {
        ObjectKey::String(key.into())
    }
}

impl From<ObjectKey> for String {
    fn from(key: ObjectKey) -> Self {
        key.to_string()
    }
}

impl Display for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectKey::Identifier(k) | ObjectKey::String(k) => Display::fmt(k, f),
            ObjectKey::RawExpression(raw) => Display::fmt(raw, f),
        }
    }
}

/// A type that holds the value of a raw expression.
///
/// As of now, anthing that is not a null value, a boolean, number, string, array or object is
/// treated as raw expression and is not further parsed. This includes conditionals, operations,
/// function calls, for expressions and variable expressions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawExpression(String);

impl RawExpression {
    /// Creates a new `RawExpression` from something that can be converted to a `String`.
    pub fn new<E>(expr: E) -> RawExpression
    where
        E: Into<String>,
    {
        RawExpression(expr.into())
    }

    /// Consumes `self` and returns the `RawExpression` as a `String`. If you want to represent the
    /// `RawExpression` as an interpolated string, use `.to_string()` instead.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for RawExpression {
    fn from(expr: String) -> Self {
        RawExpression::new(expr)
    }
}

impl From<&str> for RawExpression {
    fn from(expr: &str) -> Self {
        RawExpression::new(expr)
    }
}

impl<'a> From<Cow<'a, str>> for RawExpression {
    fn from(expr: Cow<'a, str>) -> Self {
        RawExpression::new(expr)
    }
}

impl From<RawExpression> for String {
    fn from(expr: RawExpression) -> Self {
        expr.to_string()
    }
}

impl Display for RawExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("${")?;
        f.write_str(&self.0)?;
        f.write_char('}')
    }
}
