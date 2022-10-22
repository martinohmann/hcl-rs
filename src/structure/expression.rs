//! Types to represent HCL attribute value expressions.

use super::*;
use crate::{format, Number};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Display, Write};

/// The object type used in the expression sub-language.
pub type Object<K, V> = vecmap::VecMap<K, V>;

/// A type representing the expression sub-language is used within attribute definitions to specify
/// values.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename = "$hcl::expression")]
#[non_exhaustive]
pub enum Expression {
    /// Represents a null value.
    Null,
    /// Represents a boolean.
    Bool(bool),
    /// Represents a number, either integer or float.
    Number(Number),
    /// Represents a string that does not contain any template interpolations or template
    /// directives.
    String(String),
    /// Represents array.
    Array(Vec<Expression>),
    /// Represents an object.
    Object(Object<ObjectKey, Expression>),
    /// A quoted string or heredoc that embeds a program written in the template sub-language.
    TemplateExpr(Box<TemplateExpr>),
    /// Represents a variable name identifier.
    Variable(Identifier),
    /// Represents an attribute or element traversal.
    Traversal(Box<Traversal>),
    /// Represents a function call.
    FuncCall(Box<FuncCall>),
    /// Represents a sub-expression that is wrapped in parenthesis.
    Parenthesis(Box<Expression>),
    /// A conditional operator which selects one of two rexpressions based on the outcome of a
    /// boolean expression.
    Conditional(Box<Conditional>),
    /// An operation which applies a particular operator to either one or two expression terms.
    Operation(Box<Operation>),
    /// A construct for constructing a collection by projecting the items from another collection.
    ForExpr(Box<ForExpr>),
    /// Represents a raw HCL expression. This variant will never be emitted by the parser. See
    /// [`RawExpression`] for more details.
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
            Expression::TemplateExpr(expr) => Value::String(expr.to_string()),
            Expression::Parenthesis(expr) => Value::from(*expr),
            Expression::Raw(raw) => Value::String(raw.into()),
            other => Value::String(RawExpression(other.to_string()).into()),
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
        From::from(f as f64)
    }
}

impl From<f64> for Expression {
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(Expression::Null, Expression::Number)
    }
}

impl From<Number> for Expression {
    fn from(num: Number) -> Self {
        Expression::Number(num)
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

impl From<Traversal> for Expression {
    fn from(traversal: Traversal) -> Self {
        Expression::Traversal(Box::new(traversal))
    }
}

impl From<FuncCall> for Expression {
    fn from(func_call: FuncCall) -> Self {
        Expression::FuncCall(Box::new(func_call))
    }
}

impl From<Conditional> for Expression {
    fn from(cond: Conditional) -> Self {
        Expression::Conditional(Box::new(cond))
    }
}

impl From<Operation> for Expression {
    fn from(op: Operation) -> Self {
        Expression::Operation(Box::new(op))
    }
}

impl From<ForExpr> for Expression {
    fn from(expr: ForExpr) -> Self {
        Expression::ForExpr(Box::new(expr))
    }
}

impl From<TemplateExpr> for Expression {
    fn from(expr: TemplateExpr) -> Self {
        Expression::TemplateExpr(Box::new(expr))
    }
}

impl From<Identifier> for Expression {
    fn from(ident: Identifier) -> Self {
        Expression::Variable(ident)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format::to_string_unchecked(self);
        f.write_str(&s)
    }
}

/// Represents an object key.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename = "$hcl::object_key")]
#[non_exhaustive]
pub enum ObjectKey {
    /// Represents an unquoted identifier used as object key.
    Identifier(Identifier),
    /// Any valid HCL expression can be an object key.
    Expression(Expression),
}

impl ObjectKey {
    /// Creates an unquoted string identifier `ObjectKey`.
    pub fn identifier<I>(identifier: I) -> Self
    where
        I: Into<Identifier>,
    {
        ObjectKey::Identifier(identifier.into())
    }
}

impl<T> From<T> for ObjectKey
where
    T: Into<Expression>,
{
    fn from(value: T) -> Self {
        ObjectKey::Expression(value.into())
    }
}

impl From<ObjectKey> for String {
    fn from(key: ObjectKey) -> Self {
        key.to_string()
    }
}

impl From<ObjectKey> for Value {
    fn from(key: ObjectKey) -> Self {
        match key {
            ObjectKey::Expression(expr) => expr.into(),
            ObjectKey::Identifier(ident) => Value::String(ident.into_inner()),
        }
    }
}

impl Display for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectKey::Identifier(ident) => Display::fmt(ident, f),
            ObjectKey::Expression(expr) => match expr {
                Expression::String(string) => Display::fmt(string, f),
                expr => Display::fmt(expr, f),
            },
        }
    }
}

/// A type that holds the value of a raw expression. It can be used to serialize arbitrary
/// HCL expressions.
///
/// *Please note*: raw expressions are not validated during serialization, so it is your
/// responsiblity to ensure that they are valid HCL.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename = "$hcl::raw_expression")]
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

    /// Returns the `RawExpression` as a `&str`. If you want to represent the `RawExpression` as
    /// an interpolated string, use `.to_string()` instead.
    pub fn as_str(&self) -> &str {
        &self.0
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
