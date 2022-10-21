//! Evaluate HCL templates and expressions.
//!
//! This module provides the [`Evaluate`] trait which enables HCL template and expression
//! evaluation. It is implemented for various types that either directly or transitively contain
//! templates or expressions that need to be evaluated.
//!
//! Additionally, the [`Context`] type is used to define variables and functions to make them
//! available during expression evaluation.
//!
//! For convenience, the [`from_str`] and [`to_string`] functions are provided which enable
//! expression evaluation during (de-)serialization directly. Check out their function docs for
//! usage examples.
//!
//! # Example
//!
//! HCL expressions can contain variables and functions which are made available through the
//! [`Context`] value passed to [`Evaluate::evaluate`][Evaluate::evaluate]. Function definitions
//! are created via the [`FuncDef`] type which contains more information in its type-level
//! documentation.
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use hcl::eval::{Context, Evaluate, FuncArgs, FuncDef, ParamType};
//! use hcl::{TemplateExpr, Value};
//!
//! // A template expression which needs to be evaluated. It needs access
//! // to the `uppercase` function and `name` variable.
//! let expr = TemplateExpr::QuotedString("Hello ${uppercase(name)}!".into());
//!
//! // A function that is made available to expressions via the `Context` value.
//! fn uppercase(args: FuncArgs) -> Result<Value, String> {
//!     // We know that there is one argument and it is of type `String`
//!     // because the function arguments are validated using the parameter
//!     // type information in the `FuncDef` before calling the function.
//!     Ok(Value::from(args[0].as_str().unwrap().to_uppercase()))
//! }
//!
//! // Create a definition for the `uppercase` function.
//! let uppercase_func = FuncDef::builder()
//!     .param(ParamType::String)
//!     .build(uppercase);
//!
//! // Create the context and add variables and functions to it.
//! let mut ctx = Context::new();
//! ctx.define_var("name", "world");
//! ctx.define_func("uppercase", uppercase_func);
//!
//! // Evaluate the expression.
//! assert_eq!(expr.evaluate(&ctx)?, "Hello WORLD!");
//! #   Ok(())
//! # }
//! ```

mod error;
mod expr;
mod func;
mod impls;
mod template;
#[cfg(test)]
mod tests;

pub use self::error::{Error, ErrorKind, EvalResult};
pub use self::func::*;
use crate::de::Deserializer;
use crate::parser;
use crate::structure::*;
use crate::template::*;
use crate::{Map, Result, Value};
use serde::{de, ser};

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
    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output>;
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
    pub fn define_var<I, T>(&mut self, name: I, value: T)
    where
        I: Into<Identifier>,
        T: Into<Value>,
    {
        self.vars.insert(name.into(), value.into());
    }

    /// Defines a function.
    ///
    /// See the documentation of the [`FuncDef`][FuncDef] type to learn about all available
    /// options for constructing a function definition.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl::eval::Context;
    /// use hcl::Value;
    /// use hcl::eval::{FuncArgs, FuncDef, ParamType};
    ///
    /// fn strlen(args: FuncArgs) -> Result<Value, String> {
    ///     // The arguments are already validated against the function
    ///     // definition's parameters, so we know that there is exactly
    ///     // one arg of type string.
    ///     Ok(Value::from(args[0].as_str().unwrap().len()))
    /// }
    ///
    /// let func_def = FuncDef::builder()
    ///     .param(ParamType::String)
    ///     .build(strlen);
    ///
    /// let mut ctx = Context::new();
    /// ctx.define_func("strlen", func_def);
    /// ```
    pub fn define_func<I>(&mut self, name: I, func: FuncDef)
    where
        I: Into<Identifier>,
    {
        self.funcs.insert(name.into(), func);
    }

    /// Lookup a variable's value.
    ///
    /// When the variable is defined in multiple parent scopes, the innermost variable's value is
    /// returned.
    fn lookup_var(&self, name: &Identifier) -> EvalResult<&Value> {
        self.var(name)
            .ok_or_else(|| self.error(ErrorKind::UndefinedVar(name.clone())))
    }

    /// Lookup a function definition.
    ///
    /// When the function is defined in multiple parent scopes, the innermost definition is
    /// returned.
    fn lookup_func(&self, name: &Identifier) -> EvalResult<&FuncDef> {
        self.func(name)
            .ok_or_else(|| self.error(ErrorKind::UndefinedFunc(name.clone())))
    }

    /// Creates an error enriched with expression information, if available.
    fn error<T>(&self, inner: T) -> Error
    where
        T: Into<ErrorKind>,
    {
        // The parent expression gives better context about the potential error location, use it
        // if available.
        match self.parent_expr().or_else(|| self.expr()) {
            Some(expr) => Error::new_with_expr(inner, Some(expr.clone())),
            None => Error::new(inner),
        }
    }

    fn var(&self, name: &Identifier) -> Option<&Value> {
        self.vars
            .get(name)
            .or_else(|| self.parent.and_then(|parent| parent.var(name)))
    }

    fn func(&self, name: &Identifier) -> Option<&FuncDef> {
        self.funcs
            .get(name)
            .or_else(|| self.parent.and_then(|parent| parent.func(name)))
    }

    fn expr(&self) -> Option<&Expression> {
        self.expr.or_else(|| self.parent.and_then(Context::expr))
    }

    fn parent_expr(&self) -> Option<&Expression> {
        self.parent.and_then(Context::expr)
    }
}

/// Deserialize an instance of type `T` from a string of HCL text and evaluate all expressions
/// using the given context.
///
/// ```
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use hcl::eval::Context;
/// use hcl::Body;
///
/// let input = r#"hello_world = "Hello, ${name}!""#;
///
/// let mut ctx = Context::new();
/// ctx.define_var("name", "Rust");
///
/// let body: Body = hcl::eval::from_str(input, &ctx)?;
///
/// let expected = Body::builder()
///     .add_attribute(("hello_world", "Hello, Rust!"))
///     .build();
///
/// assert_eq!(body, expected);
/// #   Ok(())
/// # }
/// ```
pub fn from_str<'de, T>(s: &str, ctx: &Context) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    let body = parser::parse(s)?;
    let evaluated = body.evaluate(ctx)?;
    let deserializer = Deserializer::from_body(evaluated);
    T::deserialize(deserializer)
}

/// Serialize the given value as an HCL string after evaulating all expressions using the given
/// context.
///
/// ```
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use hcl::eval::Context;
/// use hcl::{Body, TemplateExpr};
///
/// let expr = TemplateExpr::QuotedString("Hello, ${name}!".into());
///
/// let body = Body::builder()
///     .add_attribute(("hello_world", expr))
///     .build();
///
/// let mut ctx = Context::new();
/// ctx.define_var("name", "Rust");
///
/// let string = hcl::eval::to_string(&body, &ctx)?;
///
/// assert_eq!(string, "hello_world = \"Hello, Rust!\"\n");
/// #   Ok(())
/// # }
/// ```
pub fn to_string<T>(value: &T, ctx: &Context) -> Result<String>
where
    T: ?Sized + Evaluate,
    <T as Evaluate>::Output: ser::Serialize,
{
    let evaluated = value.evaluate(ctx)?;
    super::to_string(&evaluated)
}
