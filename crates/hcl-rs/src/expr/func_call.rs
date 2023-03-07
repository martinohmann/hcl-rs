use super::Expression;
use crate::Identifier;
use serde::Deserialize;

/// Represents a function call expression with zero or more arguments.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncCall {
    /// The name of the function.
    pub name: Identifier,
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
        T: Into<Identifier>,
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
        T: Into<Identifier>,
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
