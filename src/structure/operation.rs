use super::{de::FromStrVisitor, Expression};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Operations apply a particular operator to either one or two expression terms.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename = "$hcl::operation")]
pub enum Operation {
    /// Represents an operation that applies an operator to a single expression.
    Unary(UnaryOp),
    /// Represents an operation that applies an operator to two expressions.
    Binary(BinaryOp),
}

impl From<UnaryOp> for Operation {
    fn from(op: UnaryOp) -> Self {
        Operation::Unary(op)
    }
}

impl From<BinaryOp> for Operation {
    fn from(op: BinaryOp) -> Self {
        Operation::Binary(op)
    }
}

/// An operation that applies an operator to one expression.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename = "$hcl::unary_op")]
pub struct UnaryOp {
    /// The unary operator to use on the expression.
    pub operator: UnaryOperator,
    /// An expression that supports evaluation with the unary operator.
    pub expr: Expression,
}

impl UnaryOp {
    /// Creates a new `UnaryOp` from an operator and an expression.
    pub fn new<T>(operator: UnaryOperator, expr: T) -> UnaryOp
    where
        T: Into<Expression>,
    {
        UnaryOp {
            operator,
            expr: expr.into(),
        }
    }
}

/// An operator that can be applied to an expression.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOperator {
    /// Negate operator (`-`).
    Neg,
    /// Not operator (`!`).
    Not,
}

impl UnaryOperator {
    /// Returns the `UnaryOperator` as a static `&str`.
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOperator::Neg => "-",
            UnaryOperator::Not => "!",
        }
    }
}

impl FromStr for UnaryOperator {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "-" => Ok(UnaryOperator::Neg),
            "!" => Ok(UnaryOperator::Not),
            _ => Err(Error::new(format!("invalid unary operator: `{}`", s))),
        }
    }
}

impl Serialize for UnaryOperator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for UnaryOperator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(FromStrVisitor::<Self>::new("a unary operator"))
    }
}

/// An operation that applies an operator to two expressions.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename = "$hcl::binary_op")]
pub struct BinaryOp {
    /// The expression on the left-hand-side of the operation.
    pub lhs_expr: Expression,
    /// The binary operator to use on the expressions.
    pub operator: BinaryOperator,
    /// The expression on the right-hand-side of the operation.
    pub rhs_expr: Expression,
}

impl BinaryOp {
    /// Creates a new `BinaryOp` from two expressions and an operator.
    pub fn new<L, R>(lhs_expr: L, operator: BinaryOperator, rhs_expr: R) -> BinaryOp
    where
        L: Into<Expression>,
        R: Into<Expression>,
    {
        BinaryOp {
            lhs_expr: lhs_expr.into(),
            operator,
            rhs_expr: rhs_expr.into(),
        }
    }
}

/// An operator that can be applied to two expressions.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    /// Equal operator (`==`).
    Eq,
    /// Not-equal operator (`!=`).
    NotEq,
    /// Less-equal operator (`<=`).
    LessEq,
    /// Greater-equal operator (`>=`).
    GreaterEq,
    /// Less operator (`<`).
    Less,
    /// Greater operator (`>`).
    Greater,
    /// Plus operator (`+`).
    Plus,
    /// Minus operator (`-`).
    Minus,
    /// Multiply operator (`*`).
    Mul,
    /// Division operator (`/`).
    Div,
    /// Modulo operator (`%`).
    Mod,
    /// And operator (`&&`).
    And,
    /// Or operator (`||`).
    Or,
}

impl BinaryOperator {
    /// Returns the `BinaryOperator` as a static `&str`.
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOperator::Eq => "==",
            BinaryOperator::NotEq => "!=",
            BinaryOperator::LessEq => "<=",
            BinaryOperator::GreaterEq => ">=",
            BinaryOperator::Less => "<",
            BinaryOperator::Greater => ">",
            BinaryOperator::Plus => "+",
            BinaryOperator::Minus => "-",
            BinaryOperator::Mul => "*",
            BinaryOperator::Div => "/",
            BinaryOperator::Mod => "%",
            BinaryOperator::And => "&&",
            BinaryOperator::Or => "||",
        }
    }
}

impl FromStr for BinaryOperator {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "==" => Ok(BinaryOperator::Eq),
            "!=" => Ok(BinaryOperator::NotEq),
            "<=" => Ok(BinaryOperator::LessEq),
            ">=" => Ok(BinaryOperator::GreaterEq),
            "<" => Ok(BinaryOperator::Less),
            ">" => Ok(BinaryOperator::Greater),
            "+" => Ok(BinaryOperator::Plus),
            "-" => Ok(BinaryOperator::Minus),
            "*" => Ok(BinaryOperator::Mul),
            "/" => Ok(BinaryOperator::Div),
            "%" => Ok(BinaryOperator::Mod),
            "&&" => Ok(BinaryOperator::And),
            "||" => Ok(BinaryOperator::Or),
            _ => Err(Error::new(format!("invalid binary operator: `{}`", s))),
        }
    }
}

impl Serialize for BinaryOperator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for BinaryOperator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(FromStrVisitor::<Self>::new("a binary operator"))
    }
}
