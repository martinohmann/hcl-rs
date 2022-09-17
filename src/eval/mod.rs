//! HCL expression evaluation.

mod impls;
#[cfg(test)]
mod tests;

use crate::{BinaryOperator, Error, Expression, Map, Result, Value};
use std::fmt;

mod private {
    pub trait Sealed {}
}

pub type EvalResult<T> = Result<T, EvalError>;

#[derive(Debug)]
pub struct EvalError {
    inner: Box<EvalErrorKind>,
}

impl EvalError {
    pub fn new(inner: EvalErrorKind) -> Self {
        EvalError {
            inner: Box::new(inner),
        }
    }

    pub fn kind(&self) -> &EvalErrorKind {
        &self.inner
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<&str> for EvalError {
    fn from(msg: &str) -> Self {
        EvalError::from(msg.to_string())
    }
}

impl From<String> for EvalError {
    fn from(msg: String) -> Self {
        EvalError::new(EvalErrorKind::Message(msg))
    }
}

impl From<Error> for EvalError {
    fn from(err: Error) -> Self {
        EvalError::from(err.to_string())
    }
}

impl std::error::Error for EvalError {}

#[derive(Debug)]
#[non_exhaustive]
pub enum EvalErrorKind {
    Message(String),
    UndefinedVariable(String),
    UnexpectedExpression(Expression, &'static str),
    IndexOutOfBounds(usize),
    InvalidBinaryOp(Expression, BinaryOperator, Expression),
    NoSuchKey(String),
}

impl fmt::Display for EvalErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalErrorKind::Message(msg) => f.write_str(msg),
            EvalErrorKind::UndefinedVariable(ident) => {
                write!(f, "undefined variable `{}`", ident.as_str())
            }
            EvalErrorKind::UnexpectedExpression(expr, expected) => {
                write!(
                    f,
                    "unexpected expression `{:?}`, expected {}",
                    expr, expected
                )
            }
            EvalErrorKind::IndexOutOfBounds(index) => write!(f, "index out of bounds: {}", index),
            EvalErrorKind::NoSuchKey(key) => write!(f, "no such key: `{}`", key),
            EvalErrorKind::InvalidBinaryOp(lhs, operator, rhs) => write!(
                f,
                "operator `{}` is not applicable to `{}` and `{}`",
                operator.as_str(),
                lhs,
                rhs
            ),
        }
    }
}

/// A trait for evaluating HCL expressions.
pub trait Evaluate: private::Sealed {
    /// The type that is returned by `evaluate` on success.
    type Output;

    /// Recursively evaluates HCL expressions and returns a result which does not contain any
    /// unevaluated expressions anymore.
    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output>;
}

// @TODO(mohmann): support functions as well.
/// The evaluation context.
#[derive(Debug, Clone)]
pub struct Context {
    vars: Map<String, Value>,
}

impl Default for Context {
    fn default() -> Self {
        Context::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Context { vars: Map::new() }
    }
}

impl Context {
    fn get_variable(&self, name: &str) -> EvalResult<&Value> {
        match self.vars.get(name) {
            Some(value) => Ok(value),
            None => Err(EvalError::new(EvalErrorKind::UndefinedVariable(
                name.to_string(),
            ))),
        }
    }
}
