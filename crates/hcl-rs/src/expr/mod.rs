//! Types to represent the HCL expression sub-language.
//!
//! The module contains the [`Expression`] enum which can represent any valid HCL expression in
//! HCL attribute values and templates.

mod conditional;
pub(crate) mod de;
mod edit;
mod for_expr;
mod func_call;
mod operation;
pub(crate) mod ser;
mod template_expr;
mod traversal;
mod variable;

use self::ser::ExpressionSerializer;
pub use self::{
    conditional::Conditional,
    for_expr::ForExpr,
    func_call::{FuncCall, FuncCallBuilder, FuncName},
    operation::{BinaryOp, BinaryOperator, Operation, UnaryOp, UnaryOperator},
    template_expr::{Heredoc, HeredocStripMode, TemplateExpr},
    traversal::{Traversal, TraversalBuilder, TraversalOperator},
    variable::Variable,
};
use crate::ser::with_internal_serialization;
use crate::{format, Error, Identifier, Number, Result, Value};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Display};
use std::str::FromStr;

/// The object type used in the expression sub-language.
pub type Object<K, V> = vecmap::VecMap<K, V>;

/// A type representing the expression sub-language. It is used in HCL attributes to specify
/// values and in HCL templates.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    Variable(Variable),
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
}

impl Expression {
    #[doc(hidden)]
    pub fn from_serializable<T>(value: &T) -> Result<Expression>
    where
        T: ?Sized + Serialize,
    {
        with_internal_serialization(|| value.serialize(ExpressionSerializer))
    }
}

impl FromStr for Expression {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let expr: hcl_edit::expr::Expression = s.parse()?;
        Ok(expr.into())
    }
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
            other => Value::String(format::to_interpolated_string(&other).unwrap()),
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

impl From<UnaryOp> for Expression {
    fn from(op: UnaryOp) -> Self {
        Expression::from(Operation::Unary(op))
    }
}

impl From<BinaryOp> for Expression {
    fn from(op: BinaryOp) -> Self {
        Expression::from(Operation::Binary(op))
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

impl From<Heredoc> for Expression {
    fn from(heredoc: Heredoc) -> Self {
        Expression::from(TemplateExpr::Heredoc(heredoc))
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        Expression::Variable(variable)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formatting an `Expression` as string cannot fail.
        let formatted =
            format::to_string(self).expect("an Expression failed to format unexpectedly");
        f.write_str(&formatted)
    }
}

/// Represents an object key.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObjectKey {
    /// Represents an unquoted identifier used as object key.
    Identifier(Identifier),
    /// Any valid HCL expression can be an object key.
    Expression(Expression),
}

impl<T> From<T> for ObjectKey
where
    T: Into<Expression>,
{
    fn from(value: T) -> Self {
        ObjectKey::Expression(value.into())
    }
}

impl From<Identifier> for ObjectKey {
    fn from(ident: Identifier) -> Self {
        ObjectKey::Identifier(ident)
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

/// Convert a `T` into `hcl::Expression` which is an enum that can represent any valid HCL
/// attribute value expression.
///
/// # Errors
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
pub fn to_expression<T>(value: T) -> Result<Expression>
where
    T: Serialize,
{
    Expression::from_serializable(&value)
}
