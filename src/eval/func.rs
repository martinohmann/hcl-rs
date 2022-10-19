use super::*;
use std::fmt;
use std::iter;
use std::ops;
use std::slice;

/// A type alias for the signature of functions expected by the [`FuncDef`] type.
pub type Func = fn(FuncArgs) -> Result<Value>;

/// A type hint for a function parameter.
///
/// The parameter type is used to validate the arguments of a function call expression before
/// evaluating the function.
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl::{eval::ParamType, Value};
    /// let string_array = Value::from_iter(["foo", "bar"]);
    /// let number_array = Value::from_iter([1, 2, 3]);
    ///
    /// let param_type = ParamType::array_of(ParamType::String);
    ///
    /// assert!(param_type.is_satisfied_by(&string_array));
    /// assert!(!param_type.is_satisfied_by(&number_array));
    /// ```
    pub fn array_of(element: ParamType) -> Self {
        ParamType::Array(Box::new(element))
    }

    /// Creates a new `Object` parameter type with the given element type.
    ///
    /// The object key type is always a string and thus not specified here.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl::{eval::ParamType, Value};
    /// let object_of_strings = Value::from_iter([("foo", "bar"), ("baz", "qux")]);
    /// let object_of_numbers = Value::from_iter([("foo", 1), ("bar", 2)]);
    ///
    /// let param_type = ParamType::object_of(ParamType::String);
    ///
    /// assert!(param_type.is_satisfied_by(&object_of_strings));
    /// assert!(!param_type.is_satisfied_by(&object_of_numbers));
    /// ```
    pub fn object_of(element: ParamType) -> Self {
        ParamType::Object(Box::new(element))
    }

    /// Creates a new `OneOf` parameter type from the provided alternatives.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl::{eval::ParamType, Value};
    /// let string = Value::from("a string");
    /// let number = Value::from(42);
    /// let boolean = Value::from(true);
    ///
    /// let param_type = ParamType::one_of([ParamType::String, ParamType::Number]);
    ///
    /// assert!(param_type.is_satisfied_by(&string));
    /// assert!(param_type.is_satisfied_by(&number));
    /// assert!(!param_type.is_satisfied_by(&boolean));
    /// ```
    pub fn one_of<I>(alternatives: I) -> Self
    where
        I: IntoIterator<Item = ParamType>,
    {
        ParamType::OneOf(alternatives.into_iter().collect())
    }

    /// Creates a new `Nullable` parameter type from a non-null parameter type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl::{eval::ParamType, Value};
    /// let string = Value::from("a string");
    /// let number = Value::from(42);
    ///
    /// let param_type = ParamType::nullable(ParamType::String);
    ///
    /// assert!(param_type.is_satisfied_by(&string));
    /// assert!(param_type.is_satisfied_by(&Value::Null));
    /// assert!(!param_type.is_satisfied_by(&number));
    /// ```
    pub fn nullable(non_null: ParamType) -> Self {
        ParamType::Nullable(Box::new(non_null))
    }

    /// Tests the given value against the parameter type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl::{eval::ParamType, Value};
    /// let string = Value::from("a string");
    /// let number = Value::from(42);
    ///
    /// let param_type = ParamType::String;
    ///
    /// assert!(param_type.is_satisfied_by(&string));
    /// assert!(!param_type.is_satisfied_by(&number));
    ///
    /// let param_type = ParamType::Any;
    ///
    /// assert!(param_type.is_satisfied_by(&string));
    /// assert!(param_type.is_satisfied_by(&number));
    /// ```
    pub fn is_satisfied_by(&self, value: &Value) -> bool {
        match self {
            ParamType::Any => true,
            ParamType::Bool => value.is_boolean(),
            ParamType::Number => value.is_number(),
            ParamType::String => value.is_string(),
            ParamType::Array(elem_type) => value
                .as_array()
                .map(|array| array.iter().all(|elem| elem_type.is_satisfied_by(elem)))
                .unwrap_or(false),
            ParamType::Object(elem_type) => value
                .as_object()
                .map(|object| object.values().all(|elem| elem_type.is_satisfied_by(elem)))
                .unwrap_or(false),
            ParamType::Nullable(elem_type) => value.is_null() || elem_type.is_satisfied_by(value),
            ParamType::OneOf(elem_types) => elem_types
                .iter()
                .any(|elem_type| elem_type.is_satisfied_by(value)),
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
                            f.write_str(" or ")?;
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

/// A function parameter.
#[derive(Debug, Clone)]
pub struct Param {
    name: Identifier,
    type_: ParamType,
}

impl Param {
    /// Creates a new function parameter from a name and a type.
    pub fn new<I>(name: I, type_: ParamType) -> Self
    where
        I: Into<Identifier>,
    {
        Param {
            name: name.into(),
            type_,
        }
    }

    /// Tests the given value against the parameter's type.
    pub fn is_satisfied_by(&self, value: &Value) -> bool {
        self.type_.is_satisfied_by(value)
    }
}

impl<I, T> From<(I, T)> for Param
where
    I: Into<Identifier>,
    T: Into<ParamType>,
{
    fn from((name, type_): (I, T)) -> Self {
        Param::new(name, type_.into())
    }
}

/// The definition of a function that can be called in expressions and templates.
///
/// # Examples
///
/// ```
/// use hcl::eval::{FuncArgs, FuncDef, Param, ParamType, Result};
/// use hcl::Value;
///
/// fn add(args: FuncArgs) -> Result<Value> {
///     let a = args[0].as_number().unwrap();
///     let b = args[1].as_number().unwrap();
///     Ok(Value::Number(*a + *b))
/// }
///
/// let params = vec![
///     Param::new("a", ParamType::Number),
///     Param::new("b", ParamType::Number)
/// ];
///
/// let func_def = FuncDef::new("add", add, params);
/// ```
///
/// Alternatively, the [`FuncDefBuilder`] can be used to construct the `FuncDef`:
///
/// ```
/// # use hcl::eval::{FuncArgs, FuncDef, Param, ParamType, Result};
/// # use hcl::Value;
/// # fn add(args: FuncArgs) -> Result<Value> {
/// #    unimplemented!()
/// # }
/// let func_def = FuncDef::builder("add")
///     .param(("a", ParamType::Number))
///     .param(("b", ParamType::Number))
///     .build(add);
/// ```
///
/// See the documentation of the [`FuncDefBuilder`] for all available methods.
#[derive(Debug, Clone)]
pub struct FuncDef {
    name: Identifier,
    func: Func,
    params: Vec<Param>,
    variadic_param: Option<Param>,
}

impl FuncDef {
    /// Creates a new `FuncDef` from a name, function and it parameters.
    ///
    /// **Note**: if you want to define a `FuncDef` with a variadic parameter, use the
    /// [`.builder()`] method. It provides a [`FuncDefBuilder`] which also lets you define
    /// variadic parameters.
    ///
    /// See the type-level documentation of [`FuncDef`] for usage examples.
    ///
    /// [`.builder()`]: FuncDef::builder
    pub fn new<I, P>(name: I, func: Func, params: P) -> FuncDef
    where
        I: Into<Identifier>,
        P: IntoIterator,
        P::Item: Into<Param>,
    {
        FuncDef::builder(name).params(params).build(func)
    }

    /// Creates a [`FuncDefBuilder`].
    ///
    /// See the type-level documentation of [`FuncDef`] for usage examples.
    pub fn builder<I>(name: I) -> FuncDefBuilder
    where
        I: Into<Identifier>,
    {
        FuncDefBuilder {
            name: name.into(),
            params: Vec::new(),
            variadic_param: None,
        }
    }

    /// Returns a reference to the function name.
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns a reference to the function parameters.
    pub fn params(&self) -> &[Param] {
        &self.params
    }

    /// Returns a reference to the function's variadic parameter, or `None` if none is defined.
    pub fn variadic_param(&self) -> Option<&Param> {
        self.variadic_param.as_ref()
    }

    /// Calls the function with the provided arguments.
    ///
    /// The arguments are validated against the defined function parameters. An error is returned
    /// if too few or too many arguments are provided, of if their types do not match the expected
    /// parameter types.
    ///
    /// If all arguments are valid, the function is called and the function result returned.
    ///
    /// Because all arguments are validated before calling the function, unnecessary length and
    /// type checks on the function arguments can be avoided inside of the function.
    ///
    /// # Examples
    ///
    /// ```
    /// use hcl::eval::{FuncArgs, FuncDef, Param, ParamType, Result};
    /// use hcl::Value;
    ///
    /// fn add(args: FuncArgs) -> Result<Value> {
    ///     let a = args[0].as_number().unwrap();
    ///     let b = args[1].as_number().unwrap();
    ///     Ok(Value::Number(*a + *b))
    /// }
    ///
    /// let func_def = FuncDef::builder("add")
    ///     .param(("a", ParamType::Number))
    ///     .param(("b", ParamType::Number))
    ///     .build(add);
    ///
    /// assert!(func_def.call(["a", "b"]).is_err());
    /// assert!(func_def.call([1]).is_err());
    /// assert_eq!(func_def.call([1, 2]).unwrap(), Value::from(3));
    /// ```
    pub fn call<I>(&self, args: I) -> Result<Value>
    where
        I: IntoIterator,
        I::Item: Into<Value>,
    {
        let args: Vec<_> = args.into_iter().map(Into::into).collect();
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
            if !param.is_satisfied_by(arg) {
                return Err(self.error(format!(
                    "expected argument `{}` at position {} to be of type {}, got `{}`",
                    param.name, param.type_, pos, arg
                )));
            }
        }

        if let Some(var_param) = &var_param {
            for (pos, arg) in var_args.iter().enumerate() {
                if !var_param.is_satisfied_by(arg) {
                    return Err(self.error(format!(
                        "expected variadic argument `{}` at position {} to be of type {}, got `{}`",
                        var_param.name,
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

    fn error<T>(&self, msg: T) -> Error
    where
        T: fmt::Display,
    {
        Error::new(ErrorKind::FuncCall(self.name.clone(), msg.to_string()))
    }
}

/// A builder for `FuncDef` values.
///
/// The builder is created by the [`.builder()`] method of `FuncDef`.
///
/// See the type-level documentation of [`FuncDef`] and builder method docs for usage examples.
///
/// [`.builder()`]: FuncDef::builder
#[derive(Debug)]
pub struct FuncDefBuilder {
    name: Identifier,
    params: Vec<Param>,
    variadic_param: Option<Param>,
}

impl FuncDefBuilder {
    /// Adds a function parameter.
    ///
    /// Calls to `.param()` and [`.params()`] can be mixed and will always add more parameters to
    /// the function definition instead of overwriting existing ones.
    ///
    /// [`.params()`]: FuncDefBuilder::params
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl::eval::{FuncArgs, FuncDef, Param, ParamType, Result};
    /// # use hcl::Value;
    /// # fn strlen(_: FuncArgs) -> Result<Value> {
    /// #     unimplemented!()
    /// # }
    /// let func_def = FuncDef::builder("strlen")
    ///     .param(("string", ParamType::String))
    ///     .build(strlen);
    /// ```
    pub fn param<P>(mut self, param: P) -> FuncDefBuilder
    where
        P: Into<Param>,
    {
        self.params.push(param.into());
        self
    }

    /// Adds function parameters from an iterator.
    ///
    /// Calls to `.params()` and [`.param()`] can be mixed and will always add more parameters to
    /// the function definition instead of overwriting existing ones.
    ///
    /// [`.param()`]: FuncDefBuilder::param
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl::eval::{FuncArgs, FuncDef, Param, ParamType, Result};
    /// # use hcl::Value;
    /// # fn add3(_: FuncArgs) -> Result<Value> {
    /// #     unimplemented!()
    /// # }
    /// let func_def = FuncDef::builder("add3")
    ///     .params([
    ///         ("a", ParamType::Number),
    ///         ("b", ParamType::Number),
    ///         ("c", ParamType::Number),
    ///     ])
    ///     .build(add3);
    /// ```
    pub fn params<I>(mut self, params: I) -> FuncDefBuilder
    where
        I: IntoIterator,
        I::Item: Into<Param>,
    {
        self.params.extend(params.into_iter().map(Into::into));
        self
    }

    /// Adds a variadic parameter to the function definition.
    ///
    /// Only one variadic parameter can be added. Subsequent invocation of this method will
    /// overwrite a previously set variadic parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hcl::eval::{FuncArgs, FuncDef, Param, ParamType, Result};
    /// # use hcl::Value;
    /// # fn printf(_: FuncArgs) -> Result<Value> {
    /// #     unimplemented!()
    /// # }
    /// let func_def = FuncDef::builder("printf")
    ///     .param(("format", ParamType::String))
    ///     .variadic_param(("args", ParamType::Any))
    ///     .build(printf);
    /// ```
    pub fn variadic_param<P>(mut self, param: P) -> FuncDefBuilder
    where
        P: Into<Param>,
    {
        self.variadic_param = Some(param.into());
        self
    }

    /// Takes ownership of the builder and builds the `FuncDef` for the provided function and the
    /// contents of the builder.
    pub fn build(self, func: Func) -> FuncDef {
        FuncDef {
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
