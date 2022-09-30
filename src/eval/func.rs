use super::*;

pub type FuncImpl = fn(Vec<Value>) -> EvalResult<Value>;

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
    pub fn array_of(element: ParamType) -> Self {
        ParamType::Array(Box::new(element))
    }

    pub fn object_of(element: ParamType) -> Self {
        ParamType::Object(Box::new(element))
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
        // @TODO(mohmann): validate args
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

    pub fn build(self, func: FuncImpl) -> Func {
        Func {
            name: self.name,
            func,
            params: self.params,
            variadic_param: self.variadic_param,
        }
    }
}
