//! Evaluate HCL templates and expressions.
//!
//! This module provides the [`Evaluate`] trait which enables HCL template and expression
//! evaluation. It is implemented for various types that either directly or transitively contain
//! templates or expressions that need to be evaluated.
//!
//! Additionally, the [`Context`] type is used to declare variables and functions to make them
//! available during expression evaluation.
//!
//! For convenience, the [`from_str`] and [`to_string`] functions are provided which enable
//! expression evaluation during (de-)serialization directly. Check out their function docs for
//! usage examples.
//!
//! # Examples
//!
//! HCL expressions can contain variables and functions which are made available through the
//! [`Context`] value passed to [`Evaluate::evaluate`][Evaluate::evaluate].
//!
//! Here's a short example which evaluates a template expression that contains a variable:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hcl::Value;
//! use hcl::eval::{Context, Evaluate};
//! use hcl::expr::TemplateExpr;
//!
//! let expr = TemplateExpr::from("Hello ${name}!");
//!
//! let mut ctx = Context::new();
//! ctx.declare_var("name", "World");
//!
//! assert_eq!(expr.evaluate(&ctx)?, Value::from("Hello World!"));
//! #   Ok(())
//! # }
//! ```
//!
//! Template directives like `for` loops can be evaluated as well, this time using a
//! [`Template`][crate::template::Template] instead of [`TemplateExpr`][crate::expr::TemplateExpr]:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hcl::Template;
//! use hcl::eval::{Context, Evaluate};
//! use std::str::FromStr;
//!
//! let input = r#"
//! Bill of materials:
//! %{ for item in items ~}
//! - ${item}
//! %{ endfor ~}
//! "#;
//!
//! let template = Template::from_str(input)?;
//!
//! let mut ctx = Context::new();
//! ctx.declare_var("items", vec!["time", "code", "sweat"]);
//!
//! let evaluated = r#"
//! Bill of materials:
//! - time
//! - code
//! - sweat
//! "#;
//!
//! assert_eq!(template.evaluate(&ctx)?, evaluated);
//! #   Ok(())
//! # }
//! ```
//!
//! If you need to include the literal representation of variable reference, you can escape `${`
//! with `$${`:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hcl::eval::{Context, Evaluate};
//! use hcl::Template;
//! use std::str::FromStr;
//!
//! let template = Template::from_str("Value: ${value}, escaped: $${value}")?;
//! let mut ctx = Context::new();
//! ctx.declare_var("value", 1);
//!
//! let evaluated = "Value: 1, escaped: ${value}";
//! assert_eq!(template.evaluate(&ctx)?, evaluated);
//! #   Ok(())
//! # }
//! ```
//!
//! Here's another example which evaluates some attribute expressions using [`from_str`] as
//! described in the [deserialization
//! example][crate::eval#expression-evaluation-during-de-serialization] below:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hcl::Body;
//! use hcl::eval::Context;
//!
//! let input = r#"
//! operation   = 1 + 1
//! conditional = cond ? "yes" : "no"
//! for_expr    = [for item in items: item if item <= 3]
//! "#;
//!
//! let mut ctx = Context::new();
//! ctx.declare_var("cond", true);
//! ctx.declare_var("items", vec![1, 2, 3, 4, 5]);
//!
//! let body: Body = hcl::eval::from_str(input, &ctx)?;
//!
//! let expected = Body::builder()
//!     .add_attribute(("operation", 2))
//!     .add_attribute(("conditional", "yes"))
//!     .add_attribute(("for_expr", vec![1, 2, 3]))
//!     .build();
//!
//! assert_eq!(body, expected);
//! #   Ok(())
//! # }
//! ```
//!
//! ## Function calls in expressions
//!
//! To evaluate functions calls, you need to create a function definition and make it available to
//! the evaluation context. Function definitions are created via the [`FuncDef`] type which
//! contains more information in its [type-level documentation][FuncDef].
//!
//! Here's the example from above, updated to also include a function call to make the `name`
//! uppercase:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hcl::Value;
//! use hcl::eval::{Context, Evaluate, FuncArgs, FuncDef, ParamType};
//! use hcl::expr::TemplateExpr;
//!
//! // A template expression which needs to be evaluated. It needs access
//! // to the `uppercase` function and `name` variable.
//! let expr = TemplateExpr::from("Hello ${uppercase(name)}!");
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
//! ctx.declare_var("name", "world");
//! ctx.declare_func("uppercase", uppercase_func);
//!
//! // Evaluate the expression.
//! assert_eq!(expr.evaluate(&ctx)?, Value::from("Hello WORLD!"));
//! #   Ok(())
//! # }
//! ```
//!
//! ## Expression evaluation during (de-)serialization
//!
//! It's possible to evaluate expressions directly when deserializing HCL into a Rust value, or
//! when serializing a Rust value that contains HCL expressions into HCL.
//!
//! For these use cases the convenience functions [`hcl::eval::from_str`][from_str] and
//! [`hcl::eval::to_string`][to_string] are provided. Their usage is similar to
//! [`hcl::from_str`][crate::from_str] and [`hcl::to_string`][crate::to_string] but they receive a
//! reference to a [`Context`] value as second parameter.
//!
//! Here's a deserialization example using `from_str`:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hcl::Body;
//! use hcl::eval::Context;
//!
//! let input = r#"hello_world = "Hello, ${name}!""#;
//!
//! let mut ctx = Context::new();
//! ctx.declare_var("name", "Rust");
//!
//! let body: Body = hcl::eval::from_str(input, &ctx)?;
//!
//! let expected = Body::builder()
//!     .add_attribute(("hello_world", "Hello, Rust!"))
//!     .build();
//!
//! assert_eq!(body, expected);
//! #   Ok(())
//! # }
//! ```
//!
//! And here's how expression evaluation during serialization via `to_string` works:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use hcl::Body;
//! use hcl::eval::Context;
//! use hcl::expr::TemplateExpr;
//!
//! let expr = TemplateExpr::from("Hello, ${name}!");
//!
//! let body = Body::builder()
//!     .add_attribute(("hello_world", expr))
//!     .build();
//!
//! let mut ctx = Context::new();
//! ctx.declare_var("name", "Rust");
//!
//! let string = hcl::eval::to_string(&body, &ctx)?;
//!
//! assert_eq!(string, "hello_world = \"Hello, Rust!\"\n");
//! #   Ok(())
//! # }
//! ```

mod error;
mod expr;
mod func;
mod impls;
mod template;

pub use self::error::{Error, ErrorKind, Errors, EvalResult};
pub use self::func::{
    Func, FuncArgs, FuncDef, FuncDefBuilder, ParamType, PositionalArgs, VariadicArgs,
};
use crate::expr::{
    BinaryOp, BinaryOperator, Conditional, Expression, ForExpr, FuncCall, Object, ObjectKey,
    Operation, TemplateExpr, Traversal, TraversalOperator, UnaryOp, UnaryOperator,
};
use crate::parser;
use crate::structure::{Attribute, Block, Body, Structure};
use crate::template::{
    Directive, Element, ForDirective, IfDirective, Interpolation, Strip, Template,
};
use crate::{Identifier, Map, Result, Value};
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
    /// variables and functions declared in the `Context`.
    ///
    /// See the [module-level documentation][crate::eval] for usage examples.
    ///
    /// # Errors
    ///
    /// This function fails with an error if:
    ///
    /// - an expression evaluates to a value that is not allowed in a given context, e.g. a string
    ///   occures where a boolean value is expected.
    /// - an operation is performed on values that it's not applicable to.
    /// - an undefined variable or function is encountered.
    /// - a defined function is called with unexpected arguments.
    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output>;

    /// Recursively tries to evaluate all nested expressions in place.
    ///
    /// This function does not stop at the first error but continues to evaluate expressions as far
    /// as it can.
    ///
    /// The default implementation does nothing and always returns `Ok(())`.
    ///
    /// # Errors
    ///
    /// Returns an [`Errors`] value containing one of more [`Error`]s if the evaluation of any
    /// (potentially nested) expression fails.
    ///
    /// See the errors section of [`Evaluate::evaluate`] for a list of failure modes.
    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        _ = ctx;
        Ok(())
    }
}

/// A type holding the evaluation context.
///
/// The `Context` is used to declare variables and functions that are evaluated when evaluating a
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

    /// Declare a variable from a name and a value.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl::eval::Context;
    /// let mut ctx = Context::new();
    /// ctx.declare_var("some_number", 42);
    /// ```
    pub fn declare_var<I, T>(&mut self, name: I, value: T)
    where
        I: Into<Identifier>,
        T: Into<Value>,
    {
        self.vars.insert(name.into(), value.into());
    }

    /// Declare a function from a name and a function definition.
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
    /// ctx.declare_func("strlen", func_def);
    /// ```
    pub fn declare_func<I>(&mut self, name: I, func: FuncDef)
    where
        I: Into<Identifier>,
    {
        self.funcs.insert(name.into(), func);
    }

    /// Lookup a variable's value.
    ///
    /// When the variable is declared in multiple parent scopes, the innermost variable's value is
    /// returned.
    fn lookup_var(&self, name: &Identifier) -> EvalResult<&Value> {
        self.var(name)
            .ok_or_else(|| self.error(ErrorKind::UndefinedVar(name.clone())))
    }

    /// Lookup a function definition.
    ///
    /// When the function is declared in multiple parent scopes, the innermost definition is
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
        // The parent expression gives better context about the potential error location. Use it if
        // available.
        match self.parent_expr().or(self.expr) {
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
        self.expr.or_else(|| self.parent_expr())
    }

    fn parent_expr(&self) -> Option<&Expression> {
        self.parent.and_then(Context::expr)
    }
}

/// Deserialize an instance of type `T` from a string of HCL text and evaluate all expressions
/// using the given context.
///
/// See the [module level documentation][crate::eval#expression-evaluation-during-de-serialization]
/// for a usage example.
///
/// # Errors
///
/// This function fails with an error if:
///
/// - the string `s` cannot be parsed as HCL.
/// - any condition described in the error section of the [`evaluate` method
///   documentation][Evaluate::evaluate] meets.
/// - the evaluated value cannot be deserialized as a `T`.
pub fn from_str<T>(s: &str, ctx: &Context) -> Result<T>
where
    T: de::DeserializeOwned,
{
    let body = parser::parse(s)?;
    let evaluated = body.evaluate(ctx)?;
    super::from_body(evaluated)
}

/// Serialize the given value as an HCL string after evaluating all expressions using the given
/// context.
///
/// See the [module level documentation][crate::eval#expression-evaluation-during-de-serialization]
/// for a usage example.
///
/// # Errors
///
/// This function fails with an error if any condition described in the error section of the
/// [`evaluate` method documentation][Evaluate::evaluate] meets.
pub fn to_string<T>(value: &T, ctx: &Context) -> Result<String>
where
    T: ?Sized + Evaluate,
    <T as Evaluate>::Output: ser::Serialize,
{
    let evaluated = value.evaluate(ctx)?;
    super::to_string(&evaluated)
}
