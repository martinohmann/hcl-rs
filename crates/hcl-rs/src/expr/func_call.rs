use super::Expression;
use crate::format;
use crate::Identifier;
use serde::Deserialize;
use std::fmt;

/// Type representing a (potentially namespaced) function name.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncName {
    /// The function's namespace components, if any.
    pub namespace: Vec<Identifier>,
    /// The function name.
    pub name: Identifier,
}

impl FuncName {
    /// Create a new `FuncName` from a name identifier.
    pub fn new(name: impl Into<Identifier>) -> FuncName {
        FuncName {
            namespace: Vec::new(),
            name: name.into(),
        }
    }

    /// Adds a namespace to the function name.
    pub fn with_namespace<I>(mut self, namespace: I) -> FuncName
    where
        I: IntoIterator,
        I::Item: Into<Identifier>,
    {
        self.namespace = namespace.into_iter().map(Into::into).collect();
        self
    }

    /// Returns `true` if the function name is namespaced.
    pub fn is_namespaced(&self) -> bool {
        !self.namespace.is_empty()
    }
}

impl<T> From<T> for FuncName
where
    T: Into<Identifier>,
{
    fn from(name: T) -> Self {
        FuncName {
            namespace: Vec::new(),
            name: name.into(),
        }
    }
}

impl fmt::Display for FuncName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formatting a `FuncName` as string cannot fail.
        let formatted = format::to_string(self).expect("a FuncName failed to format unexpectedly");
        f.write_str(&formatted)
    }
}

/// Represents a function call expression with zero or more arguments.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncCall {
    /// The function name.
    pub name: FuncName,
    /// The function arguments.
    pub args: Vec<Expression>,
    /// If `true`, the final argument should be an array which will expand to be one argument per
    /// element.
    pub expand_final: bool,
}

impl FuncCall {
    /// Creates a new `FuncCall` for the function with given name.
    pub fn new<T>(name: T) -> FuncCall
    where
        T: Into<FuncName>,
    {
        FuncCall {
            name: name.into(),
            args: Vec::new(),
            expand_final: false,
        }
    }

    /// Creates a new `FuncCallBuilder` for the function with given name.
    pub fn builder<T>(name: T) -> FuncCallBuilder
    where
        T: Into<FuncName>,
    {
        FuncCallBuilder {
            f: FuncCall::new(name),
        }
    }
}

/// A builder for function calls.
#[derive(Debug)]
pub struct FuncCallBuilder {
    f: FuncCall,
}

impl FuncCallBuilder {
    /// Adds an argument to the function call.
    pub fn arg<T>(mut self, arg: T) -> FuncCallBuilder
    where
        T: Into<Expression>,
    {
        self.f.args.push(arg.into());
        self
    }

    /// If `true`, the final argument should be an array which will expand to be one argument per
    /// element.
    pub fn expand_final(mut self, yes: bool) -> FuncCallBuilder {
        self.f.expand_final = yes;
        self
    }

    /// Consumes the `FuncCallBuilder` and returns the `FuncCall`.
    pub fn build(self) -> FuncCall {
        self.f
    }
}
