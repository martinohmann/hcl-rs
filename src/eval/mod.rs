//! HCL expression evaluation.

mod impls;
#[cfg(test)]
mod tests;

use crate::{BinaryOperator, BlockLabel, Error, Expression, Map, ObjectKey, Result, Value};
use std::fmt;

mod private {
    pub trait Sealed {}
}

pub type EvalResult<T> = Result<T, EvalError>;

#[derive(Debug)]
pub struct EvalError {
    inner: Box<EvalErrorKind>,
    scope: Option<String>,
}

impl EvalError {
    pub fn new(inner: EvalErrorKind) -> Self {
        EvalError {
            inner: Box::new(inner),
            scope: None,
        }
    }

    pub fn kind(&self) -> &EvalErrorKind {
        &self.inner
    }

    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    fn with_scopes(mut self, scopes: Scopes<'_>) -> EvalError {
        self.scope = Some(scopes.to_string());
        self
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(scope) = &self.scope {
            write!(f, "at {}: ", scope)?;
        }

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
    RawExpression,
    Message(String),
    UndefinedVariable(String),
    Unexpected(Expression, &'static str),
    IndexOutOfBounds(usize),
    InvalidBinaryOp(Expression, BinaryOperator, Expression),
    NoSuchKey(String),
}

impl fmt::Display for EvalErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalErrorKind::RawExpression => f.write_str("raw expressions cannot be evaluated"),
            EvalErrorKind::Message(msg) => f.write_str(msg),
            EvalErrorKind::UndefinedVariable(ident) => {
                write!(f, "undefined variable `{}`", ident.as_str())
            }
            EvalErrorKind::Unexpected(expr, expected) => {
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
pub struct Context<'a> {
    vars: Map<String, Value>,
    parent: Option<&'a Context<'a>>,
    scope: Option<Scope<'a>>,
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
            parent: None,
            scope: None,
        }
    }

    // Create a new context scope which has the current one as parent.
    fn new_scope(&'a self, scope: Scope<'a>) -> Context<'a> {
        Context {
            vars: Map::new(),
            parent: Some(self),
            scope: Some(scope),
        }
    }

    /// Lookup a variable's value. Variables defined in the current scope take precedence over
    /// variables defined in parent scopes.
    pub fn get_variable(&self, name: &str) -> EvalResult<&Value> {
        match self.vars.get(name) {
            Some(value) => Ok(value),
            None => match &self.parent {
                Some(parent) => parent.get_variable(name),
                None => Err(EvalError::new(EvalErrorKind::UndefinedVariable(
                    name.to_string(),
                ))),
            },
        }
    }

    /// Set a variable which is available in the current and all child scopes.
    pub fn set_variable<T>(&mut self, name: String, value: T) -> Option<Value>
    where
        T: Into<Value>,
    {
        self.vars.insert(name, value.into())
    }

    // Collects all scopes into a flat list.
    fn scopes(&self) -> Scopes<'_> {
        let mut ctx = self;
        let mut scopes = Vec::new();

        loop {
            if let Some(scope) = &ctx.scope {
                scopes.push(scope);
            }

            match ctx.parent {
                Some(parent) => ctx = parent,
                None => break,
            };
        }

        scopes.reverse();
        Scopes(scopes)
    }

    // Creates an `EvalError` with added scope information.
    fn error(&self, kind: EvalErrorKind) -> EvalError {
        EvalError::new(kind).with_scopes(self.scopes())
    }
}

#[derive(Debug, Clone)]
enum Scope<'a> {
    Attr(&'a str),
    Block(&'a str, &'a [BlockLabel]),
    Key(&'a ObjectKey),
    Index(usize),
    Expr(&'a Expression),
}

struct Scopes<'a>(Vec<&'a Scope<'a>>);

impl fmt::Display for Scopes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for scope in &self.0 {
            match scope {
                Scope::Block(ident, labels) => {
                    write!(f, ".{}", ident)?;
                    for label in labels.iter() {
                        match label {
                            BlockLabel::Identifier(ident) => write!(f, ".{}", ident.as_str())?,
                            BlockLabel::String(string) => write!(f, "\"{}\"", string)?,
                        }
                    }
                }
                Scope::Attr(key) => write!(f, ".{}", key)?,
                Scope::Key(key) => match key {
                    ObjectKey::Identifier(ident) => write!(f, ".{}", ident.as_str())?,
                    ObjectKey::Expression(expr) => write!(f, "[{}]", expr)?,
                },
                Scope::Index(index) => write!(f, "[{}]", index)?,
                Scope::Expr(expr) => write!(f, "= {}", expr)?,
            }
        }

        Ok(())
    }
}
