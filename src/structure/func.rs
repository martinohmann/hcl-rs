use crate::{Expression, Identifier};
use serde::{Deserialize, Serialize};

/// Represents a function call expression with zero or more arguments. Function calls can be
/// variadic.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename = "$hcl::func_call")]
pub struct FuncCall {
    /// The name of the function.
    pub name: Identifier,
    /// The function arguments.
    pub args: Vec<Expression>,
    /// Specifies whether this is a variadic function call or not.
    pub variadic: bool,
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
            variadic: false,
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

    /// Marks the function as variadic or not.
    pub fn variadic(mut self, yes: bool) -> FuncCallBuilder {
        self.f.variadic = yes;
        self
    }

    /// Consumes the `FuncCallBuilder` and returns the `FuncCall`.
    pub fn build(self) -> FuncCall {
        self.f
    }
}
