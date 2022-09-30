use super::*;
use std::fmt;

pub type FuncImpl = fn(Vec<Value>) -> EvalResult<Value>;

#[derive(Debug, Clone)]
pub enum ParamType {
    Any,
    Bool,
    Number,
    String,
    Array(Box<ParamType>),
    Object(Box<ParamType>),
    Nullable(Box<ParamType>),
}

impl ParamType {
    pub fn array_of(element: ParamType) -> Self {
        ParamType::Array(Box::new(element))
    }

    pub fn object_of(element: ParamType) -> Self {
        ParamType::Object(Box::new(element))
    }

    pub fn nullable(element: ParamType) -> Self {
        ParamType::Nullable(Box::new(element))
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
        }
    }
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamType::Any => f.write_str("any"),
            ParamType::Bool => f.write_str("bool"),
            ParamType::Number => f.write_str("number"),
            ParamType::String => f.write_str("string"),
            ParamType::Array(element) => write!(f, "array({})", element),
            ParamType::Object(element) => write!(f, "object({})", element),
            ParamType::Nullable(element) => write!(f, "nullable({})", element),
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

    pub fn call<I>(&self, args: I) -> EvalResult<Value>
    where
        I: IntoIterator,
        I::Item: Into<Value>,
    {
        let args: Vec<Value> = args.into_iter().map(Into::into).collect();

        let params_len = self.params.len();
        let pos_args = &args[..params_len];
        let var_args = &args[params_len..];

        if pos_args.len() != params_len {
            return Err(self.error(format!(
                "expected {} positional arguments, got {}",
                params_len,
                pos_args.len(),
            )));
        }

        if self.variadic_param.is_none() && !var_args.is_empty() {
            return Err(self.error(format!(
                "expected {} positional arguments, got {}",
                params_len,
                pos_args.len() + var_args.len(),
            )));
        }

        for (pos, (arg, param)) in pos_args.iter().zip(self.params.iter()).enumerate() {
            if !param.matches(arg) {
                return Err(self.error(format!(
                    "expected argument at position {} to be of type `{}`, got `{}`",
                    param.type_, pos, arg
                )));
            }
        }

        if let Some(var_param) = &self.variadic_param {
            for (pos, arg) in var_args.iter().enumerate() {
                if !var_param.matches(arg) {
                    return Err(self.error(format!(
                        "expected argument at position {} to be of type `{}`, got `{}`",
                        var_param.type_,
                        pos_args.len() + pos,
                        arg
                    )));
                }
            }
        }

        (self.func)(args)
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
