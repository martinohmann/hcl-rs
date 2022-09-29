//! HCL expression evaluation.

mod expr;
mod for_expr;
mod func;
mod impls;
mod template;
#[cfg(test)]
mod tests;

use self::func::{Func, FuncArgs};
use crate::{
    BinaryOperator, Error, Expression, Identifier, Map, ObjectKey, Result, UnaryOperator, Value,
};
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvalErrorKind {
    RawExpression,
    Message(String),
    UndefinedVariable(Identifier),
    UndefinedFunc(Identifier),
    Unexpected(Expression, &'static str),
    IndexOutOfBounds(usize),
    InvalidUnaryOp(UnaryOperator, Expression),
    InvalidBinaryOp(Expression, BinaryOperator, Expression),
    NoSuchKey(ObjectKey),
    KeyAlreadyExists(ObjectKey),
}

impl fmt::Display for EvalErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalErrorKind::RawExpression => f.write_str("raw expressions cannot be evaluated"),
            EvalErrorKind::Message(msg) => f.write_str(msg),
            EvalErrorKind::UndefinedVariable(ident) => {
                write!(f, "undefined variable `{}`", ident.as_str())
            }
            EvalErrorKind::UndefinedFunc(ident) => {
                write!(f, "undefined function `{}`", ident.as_str())
            }
            EvalErrorKind::Unexpected(expr, expected) => {
                write!(f, "unexpected expression `{}`, expected {}", expr, expected)
            }
            EvalErrorKind::IndexOutOfBounds(index) => write!(f, "index out of bounds: {}", index),
            EvalErrorKind::NoSuchKey(key) => write!(f, "no such key: `{}`", key),
            EvalErrorKind::KeyAlreadyExists(key) => write!(f, "key `{}` already exists", key),
            EvalErrorKind::InvalidUnaryOp(operator, expr) => write!(
                f,
                "unary operator `{}` is not applicable to `{}`",
                operator.as_str(),
                expr,
            ),
            EvalErrorKind::InvalidBinaryOp(lhs, operator, rhs) => write!(
                f,
                "binary operator `{}` is not applicable to `{}` and `{}`",
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
    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output>;
}

/// The evaluation context.
#[derive(Debug, Clone)]
pub struct Context<'a> {
    vars: Map<Identifier, Value>,
    funcs: Map<Identifier, Func>,
    parent: Option<&'a Context<'a>>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Context::new()
    }
}

impl<'a> Context<'a> {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Context {
            vars: Map::new(),
            funcs: Map::new(),
            parent: None,
        }
    }

    // Create a new child `Context` which has the current one as parent.
    fn new_child(&self) -> Context<'_> {
        Context {
            vars: Map::new(),
            funcs: Map::new(),
            parent: Some(self),
        }
    }

    /// Lookup a variable's value. Variables defined in the current scope take precedence over
    /// variables defined in parent scopes.
    pub fn get_var(&self, name: &Identifier) -> EvalResult<&Value> {
        match self.vars.get(name) {
            Some(value) => Ok(value),
            None => match self.parent {
                Some(parent) => parent.get_var(name),
                None => Err(EvalError::new(EvalErrorKind::UndefinedVariable(
                    name.clone(),
                ))),
            },
        }
    }

    /// Set a variable which is available in the current and all child scopes.
    pub fn set_var<I, T>(&mut self, name: I, value: T) -> Option<Value>
    where
        I: Into<Identifier>,
        T: Into<Value>,
    {
        self.vars.insert(name, value.into())
    }

    /// Lookup a func. Functions defined in the current scope take precedence over
    /// functions defined in parent scopes.
    pub fn get_func(&self, name: &Identifier) -> EvalResult<&Func> {
        match self.funcs.get(name) {
            Some(func) => Ok(func),
            None => match self.parent {
                Some(parent) => parent.get_func(name),
                None => Err(EvalError::new(EvalErrorKind::UndefinedFunc(name.clone()))),
            },
        }
    }

    /// Set a func which is available in the current and all child scopes.
    pub fn set_func<I>(&mut self, name: I, func: Func) -> Option<Func>
    where
        I: Into<Identifier>,
    {
        self.funcs.insert(name, func)
    }

    // Creates an `EvalError`.
    fn error(&self, kind: EvalErrorKind) -> EvalError {
        EvalError::new(kind)
    }
}
