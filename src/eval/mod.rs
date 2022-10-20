//! HCL expression evaluation.

mod error;
mod expr;
mod func;
mod impls;
mod template;
#[cfg(test)]
mod tests;

pub use self::error::{Error, ErrorKind, Result};
pub use self::func::*;
use crate::structure::*;
use crate::template::*;
use crate::{Map, Value};

mod private {
    pub trait Sealed {}
}

/// A trait for evaluating the HCL template and expression sub-languages.
///
/// The types implementing this trait must recursively evaluate all HCL templates and expressions
/// in their fields.
///
/// This trait is sealed to prevent implementation outside of this crate.
pub trait Evaluate: private::Sealed {
    /// The type that is returned by [`evaluate`][Evaluate::evaluate] on success.
    type Output;

    /// Recursively evaluates all HCL templates and expressions in the implementing type using the
    /// variables and functions defined in the `Context`.
    fn evaluate(&self, ctx: &Context) -> Result<Self::Output>;
}

/// A type holding the evaluation context.
///
/// The `Context` is used to define variables and functions that are evaluated when evaluating a
/// template or expression.
#[derive(Debug, Clone)]
pub struct Context<'a> {
    vars: Map<Identifier, Value>,
    funcs: Map<Identifier, FuncDef>,
    parent: Option<&'a Context<'a>>,
    expr: Option<&'a Expression>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Context {
            vars: Map::new(),
            funcs: Map::new(),
            parent: None,
            expr: None,
        }
    }
}

impl<'a> Context<'a> {
    /// Creates an empty `Context`.
    pub fn new() -> Self {
        Context::default()
    }

    // Create a new child `Context` which has the current one as parent.
    fn child(&self) -> Context<'_> {
        let mut ctx = Context::new();
        ctx.parent = Some(self);
        ctx
    }

    // Create a new child `Context` which has the current one as parent and also contains context
    // about the expression that is currently evaluated.
    fn child_with_expr(&self, expr: &'a Expression) -> Context<'_> {
        let mut ctx = self.child();
        ctx.expr = Some(expr);
        ctx
    }

    /// Defines a variable.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl::eval::Context;
    /// let mut ctx = Context::new();
    /// ctx.define_var("some_number", 42);
    /// ```
    pub fn define_var<I, T>(&mut self, name: I, value: T) -> &mut Self
    where
        I: Into<Identifier>,
        T: Into<Value>,
    {
        self.vars.insert(name.into(), value.into());
        self
    }

    /// Defines a function which is available in the current and all child scopes.
    ///
    /// See the documentation of the [`FuncDef`][FuncDef] type to learn about all available
    /// options for constructing a function definition.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl::eval::Context;
    /// use hcl::Value;
    /// use hcl::eval::{FuncArgs, FuncDef, Param, ParamType, Result};
    ///
    /// fn strlen(args: FuncArgs) -> Result<Value, String> {
    ///     // The arguments are already validated against the function
    ///     // definition's parameters, so we know that there is exactly
    ///     // one arg of type string.
    ///     Ok(Value::from(args[0].as_str().unwrap().len()))
    /// }
    ///
    /// let func_def = FuncDef::builder("strlen")
    ///     .param(Param::new("s", ParamType::String))
    ///     .build(strlen);
    ///
    /// let mut ctx = Context::new();
    /// ctx.define_func(func_def);
    /// ```
    pub fn define_func(&mut self, func: FuncDef) -> &mut Self {
        self.funcs.insert(func.name().clone(), func);
        self
    }

    /// Lookup a variable's value.
    ///
    /// When the variable is defined in multiple parent scopes, the innermost variable's value is
    /// returned.
    fn lookup_var(&self, name: &Identifier) -> Result<&Value> {
        match self.vars.get(name) {
            Some(value) => Ok(value),
            None => match self.parent {
                Some(parent) => parent.lookup_var(name),
                None => Err(Error::new(ErrorKind::UndefinedVariable(name.clone()))),
            },
        }
    }

    /// Lookup a function definition.
    ///
    /// When the function is defined in multiple parent scopes, the innermost definition is
    /// returned.
    fn lookup_func(&self, name: &Identifier) -> Result<&FuncDef> {
        match self.funcs.get(name) {
            Some(func) => Ok(func),
            None => match self.parent {
                Some(parent) => parent.lookup_func(name),
                None => Err(Error::new(ErrorKind::UndefinedFunc(name.clone()))),
            },
        }
    }

    /// Creates an error enriched with expression information, if available.
    fn error<T>(&self, inner: T) -> Error
    where
        T: Into<ErrorKind>,
    {
        match self.expr() {
            Some(expr) => Error::new_with_expr(inner, Some(expr.clone())),
            None => Error::new(inner),
        }
    }

    fn expr(&self) -> Option<&Expression> {
        self.expr
            .or_else(|| self.parent.and_then(|parent| parent.expr()))
    }
}
