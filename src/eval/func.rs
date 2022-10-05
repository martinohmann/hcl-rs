use super::*;
use std::fmt;
use std::iter;
use std::ops;
use std::slice;

pub type FuncImpl = fn(FuncArgs) -> EvalResult<Value>;

/// A type hint for a function parameter.
///
/// This is used to validate the parameters of a function call expression before evaluating the
/// function.
#[derive(Debug, Clone)]
pub enum ParamType {
    /// Any type is allowed.
    Any,
    /// The parameter must be a boolean value.
    Bool,
    /// The parameter must be a number.
    Number,
    /// The parameter must be a string value.
    String,
    /// The parameter must be an array which must contain only elements of the given element type.
    Array(Box<ParamType>),
    /// The parameter must be an object which must contain only entries with values of the given
    /// element type. The object key type is always a string.
    Object(Box<ParamType>),
    /// The parameter can be one of the provided types. If the `Vec` is empty, any type is
    /// allowed.
    OneOf(Vec<ParamType>),
    /// The parameter must be either `null` or of the provided type.
    Nullable(Box<ParamType>),
}

impl ParamType {
    /// Creates a new `Array` parameter type with the given element type.
    pub fn array_of(element: ParamType) -> Self {
        ParamType::Array(Box::new(element))
    }

    /// Creates a new `Object` parameter type with the given element type.
    pub fn object_of(element: ParamType) -> Self {
        ParamType::Object(Box::new(element))
    }

    /// Creates a new `OneOf` parameter type from the provided alternatives.
    pub fn one_of<I>(alternatives: I) -> Self
    where
        I: IntoIterator<Item = ParamType>,
    {
        ParamType::OneOf(alternatives.into_iter().collect())
    }

    /// Creates a new `Nullable` parameter type from a non-null parameter type.
    pub fn nullable(non_null: ParamType) -> Self {
        ParamType::Nullable(Box::new(non_null))
    }

    fn matches(&self, value: &Value) -> bool {
        match self {
            ParamType::Any => true,
            ParamType::Bool => value.is_boolean(),
            ParamType::Number => value.is_number(),
            ParamType::String => value.is_string(),
            ParamType::Array(elem_type) => match value.as_array() {
                Some(array) => array.iter().all(|elem| elem_type.matches(elem)),
                None => false,
            },
            ParamType::Object(elem_type) => match value.as_object() {
                Some(object) => object.values().all(|elem| elem_type.matches(elem)),
                None => false,
            },
            ParamType::Nullable(elem_type) => value.is_null() || elem_type.matches(value),
            ParamType::OneOf(elem_types) => {
                elem_types.iter().any(|elem_type| elem_type.matches(value))
            }
        }
    }
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamType::Any => f.write_str("`any`"),
            ParamType::Bool => f.write_str("`bool`"),
            ParamType::Number => f.write_str("`number`"),
            ParamType::String => f.write_str("`string`"),
            ParamType::Array(elem_type) => write!(f, "`array({})`", elem_type),
            ParamType::Object(elem_type) => write!(f, "`object({})`", elem_type),
            ParamType::Nullable(elem_type) => write!(f, "`nullable({})`", elem_type),
            ParamType::OneOf(elem_types) => match elem_types.len() {
                0 => f.write_str("`any`"),
                1 => fmt::Display::fmt(&elem_types[0], f),
                n => {
                    for (i, elem_type) in elem_types.iter().enumerate() {
                        if i == n - 1 {
                            f.write_str("or ")?;
                        } else if i > 0 {
                            f.write_str(", ")?;
                        }

                        fmt::Display::fmt(elem_type, f)?;
                    }
                    Ok(())
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    name: Identifier,
    type_: ParamType,
}

impl Param {
    pub fn new<I, T>(name: I, type_: T) -> Self
    where
        I: Into<Identifier>,
        T: Into<ParamType>,
    {
        Param {
            name: name.into(),
            type_: type_.into(),
        }
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn type_(&self) -> &ParamType {
        &self.type_
    }

    fn matches(&self, value: &Value) -> bool {
        self.type_.matches(value)
    }
}

impl<I, T> From<(I, T)> for Param
where
    I: Into<Identifier>,
    T: Into<ParamType>,
{
    fn from((name, type_): (I, T)) -> Self {
        Param::new(name, type_)
    }
}

#[derive(Debug, Clone)]
pub struct Func {
    name: Identifier,
    func: FuncImpl,
    params: Vec<Param>,
    variadic_param: Option<Param>,
}

impl Func {
    pub fn new<I, P>(name: I, func: FuncImpl, params: P) -> Func
    where
        I: Into<Identifier>,
        P: IntoIterator,
        P::Item: Into<Param>,
    {
        Func::builder(name).params(params).build(func)
    }

    pub fn builder<I>(name: I) -> FuncBuilder
    where
        I: Into<Identifier>,
    {
        FuncBuilder {
            name: name.into(),
            params: Vec::new(),
            variadic_param: None,
        }
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn params(&self) -> &[Param] {
        &self.params
    }

    pub fn variadic_param(&self) -> Option<&Param> {
        self.variadic_param.as_ref()
    }

    pub fn call(&self, args: Vec<Value>) -> EvalResult<Value> {
        let params_len = self.params.len();
        let args_len = args.len();
        let var_param = &self.variadic_param;

        if args_len < params_len || (var_param.is_none() && args_len > params_len) {
            return Err(self.error(format!(
                "expected {} positional arguments, got {}",
                params_len, args_len,
            )));
        }

        let (pos_args, var_args) = args.split_at(params_len);

        for (pos, (arg, param)) in pos_args.iter().zip(self.params.iter()).enumerate() {
            if !param.matches(arg) {
                return Err(self.error(format!(
                    "expected argument at position {} to be of type {}, got `{}`",
                    param.type_, pos, arg
                )));
            }
        }

        if let Some(var_param) = &var_param {
            for (pos, arg) in var_args.iter().enumerate() {
                if !var_param.matches(arg) {
                    return Err(self.error(format!(
                        "expected variadic argument at position {} to be of type {}, got `{}`",
                        var_param.type_,
                        params_len + pos,
                        arg
                    )));
                }
            }
        }

        let func_args = FuncArgs::new(args, params_len);

        (self.func)(func_args)
    }

    fn error<T>(&self, msg: T) -> EvalError
    where
        T: fmt::Display,
    {
        EvalError::new(EvalErrorKind::FuncCall(self.name.clone(), msg.to_string()))
    }
}

#[derive(Debug)]
pub struct FuncBuilder {
    name: Identifier,
    params: Vec<Param>,
    variadic_param: Option<Param>,
}

impl FuncBuilder {
    pub fn param<P>(mut self, param: P) -> FuncBuilder
    where
        P: Into<Param>,
    {
        self.params.push(param.into());
        self
    }

    pub fn params<P>(mut self, params: P) -> FuncBuilder
    where
        P: IntoIterator,
        P::Item: Into<Param>,
    {
        self.params.extend(params.into_iter().map(Into::into));
        self
    }

    pub fn variadic_param<P>(mut self, param: P) -> FuncBuilder
    where
        P: Into<Param>,
    {
        self.variadic_param = Some(param.into());
        self
    }

    pub fn build(self, func: FuncImpl) -> Func {
        Func {
            name: self.name,
            func,
            params: self.params,
            variadic_param: self.variadic_param,
        }
    }
}

/// Wrapper type for function argument values.
///
/// During expression evaluation it is passed to functions referenced by function call
/// expressions with the values of the evaluated argument expressions.
///
/// `FuncArgs` behaves exactly like a `Vec<Value>` due to its `Deref` implementation, but exposes
/// additional methods to iterate over positional and variadic arguments.
#[derive(Debug, Clone)]
pub struct FuncArgs {
    values: Vec<Value>,
    pos_args_len: usize,
}

impl FuncArgs {
    pub(super) fn new(values: Vec<Value>, pos_args_len: usize) -> FuncArgs {
        FuncArgs {
            values,
            pos_args_len,
        }
    }

    /// Takes ownership of the function argument values.
    pub fn into_values(self) -> Vec<Value> {
        self.values
    }

    /// Returns an iterator over all positional arguments.
    pub fn positional_args(&self) -> PositionalArgs<'_> {
        PositionalArgs {
            iter: self.values.iter().take(self.pos_args_len),
        }
    }

    /// Returns an iterator over all variadic arguments.
    pub fn variadic_args(&self) -> VariadicArgs<'_> {
        VariadicArgs {
            iter: self.values.iter().skip(self.pos_args_len),
        }
    }
}

impl ops::Deref for FuncArgs {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

/// An iterator over positional function arguments.
///
/// This `struct` is created by the [`positional_args`] method on [`FuncArgs`]. See its
/// documentation for more.
///
/// [`positional_args`]: FuncArgs::positional_args
#[derive(Debug, Clone)]
pub struct PositionalArgs<'a> {
    iter: iter::Take<slice::Iter<'a, Value>>,
}

impl<'a> Iterator for PositionalArgs<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// An iterator over variadic function arguments.
///
/// This `struct` is created by the [`variadic_args`] method on [`FuncArgs`]. See its
/// documentation for more.
///
/// [`variadic_args`]: FuncArgs::variadic_args
#[derive(Debug, Clone)]
pub struct VariadicArgs<'a> {
    iter: iter::Skip<slice::Iter<'a, Value>>,
}

impl<'a> Iterator for VariadicArgs<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
