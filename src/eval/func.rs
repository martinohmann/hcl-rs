use super::*;

pub type Function = fn(FuncArgs) -> EvalResult<Value>;

#[derive(Debug, Clone)]
pub enum ParamType {
    Any,
    Bool,
    Number,
    String,
    Array(Box<ParamType>),
    Object(Box<ParamType>),
}

impl ParamType {
    pub fn array(element: ParamType) -> Self {
        ParamType::Array(Box::new(element))
    }

    pub fn object(element: ParamType) -> Self {
        ParamType::Object(Box::new(element))
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    name: Identifier,
    typ: ParamType,
}

impl Param {
    pub fn new<I, T>(name: I, typ: T) -> Self
    where
        I: Into<Identifier>,
        T: Into<ParamType>,
    {
        Param {
            name: name.into(),
            typ: typ.into(),
        }
    }
}

impl<I, T> From<(I, T)> for Param
where
    I: Into<Identifier>,
    T: Into<ParamType>,
{
    fn from((name, typ): (I, T)) -> Self {
        Param::new(name, typ)
    }
}

#[derive(Debug, Clone)]
pub struct Func {
    name: Identifier,
    func: fn(FuncArgs) -> EvalResult<Value>,
    params: Vec<Param>,
    variadic_param: Option<Param>,
}

impl Func {
    pub fn new<I, P>(name: I, func: fn(FuncArgs) -> EvalResult<Value>, params: P) -> Func
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
        let args = FuncArgs::new(args);
        (self.func)(args)
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

    pub fn build(self, func: fn(FuncArgs) -> EvalResult<Value>) -> Func {
        Func {
            name: self.name,
            func,
            params: self.params,
            variadic_param: self.variadic_param,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FuncArgs {
    args: Vec<Value>,
}

impl FuncArgs {
    pub fn new<I>(args: I) -> Self
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Into<Value>,
    {
        FuncArgs {
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    pub fn ensure_len(&self, count: usize) -> EvalResult<()> {
        if self.args.len() == count {
            Ok(())
        } else {
            Err(EvalError::new(EvalErrorKind::Message(format!(
                "unexpected argument count, expected {} but got {}",
                count,
                self.args.len()
            ))))
        }
    }

    pub fn ensure_min_len(&self, count: usize) -> EvalResult<()> {
        if self.args.len() >= count {
            Ok(())
        } else {
            Err(EvalError::new(EvalErrorKind::Message(format!(
                "unexpected argument count, expected at least {} but got {}",
                count,
                self.args.len()
            ))))
        }
    }

    pub fn get(&self, index: usize) -> EvalResult<&Value> {
        self.args
            .get(index)
            .ok_or_else(|| EvalError::new(EvalErrorKind::IndexOutOfBounds(index)))
    }

    pub fn get_bool(&self, index: usize) -> EvalResult<bool> {
        match self.get(index)? {
            Value::Bool(b) => Ok(*b),
            other => Err(EvalError::unexpected(other.clone(), "a boolean")),
        }
    }

    pub fn get_number(&self, index: usize) -> EvalResult<&Number> {
        match self.get(index)? {
            Value::Number(num) => Ok(num),
            other => Err(EvalError::unexpected(other.clone(), "a number")),
        }
    }

    pub fn get_str(&self, index: usize) -> EvalResult<&str> {
        match self.get(index)? {
            Value::String(string) => Ok(string),
            other => Err(EvalError::unexpected(other.clone(), "a string")),
        }
    }

    pub fn get_array(&self, index: usize) -> EvalResult<&Vec<Value>> {
        match self.get(index)? {
            Value::Array(array) => Ok(array),
            other => Err(EvalError::unexpected(other.clone(), "an array")),
        }
    }

    pub fn get_object(&self, index: usize) -> EvalResult<&Map<String, Value>> {
        match self.get(index)? {
            Value::Object(object) => Ok(object),
            other => Err(EvalError::unexpected(other.clone(), "an object")),
        }
    }

    pub fn as_slice(&self) -> &[Value] {
        &self.args
    }

    pub fn as_slice_mut(&mut self) -> &mut [Value] {
        &mut self.args
    }

    pub fn into_inner(self) -> Vec<Value> {
        self.args
    }
}
