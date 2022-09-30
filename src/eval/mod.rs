//! HCL expression evaluation.

mod error;
mod expr;
mod for_expr;
#[allow(dead_code)]
pub mod func;
mod impls;
mod template;
#[cfg(test)]
mod tests;

pub use self::error::{EvalError, EvalErrorKind, EvalResult};
use self::for_expr::Collection;
use self::func::*;
use crate::structure::*;
use crate::template::*;
use crate::{Error, Map, Result, Value};

mod private {
    pub trait Sealed {}
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
        self.vars.insert(name.into(), value.into())
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
    pub fn add_func(&mut self, func: Func) -> Option<Func> {
        self.funcs.insert(func.name().clone(), func)
    }
}
