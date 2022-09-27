use super::*;
use crate::Number;

pub type Func = fn(FuncContext) -> EvalResult<Value>;

#[derive(Debug, Clone)]
pub struct FuncContext {
    args: Vec<Value>,
}

impl FuncContext {
    pub fn new<I>(args: I) -> Self
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Into<Value>,
    {
        FuncContext {
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    pub fn ensure_exact_args(&self, count: usize) -> EvalResult<()> {
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

    pub fn ensure_min_args(&self, count: usize) -> EvalResult<()> {
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

    pub fn get_arg(&self, index: usize) -> EvalResult<&Value> {
        self.args
            .get(index)
            .ok_or_else(|| EvalError::new(EvalErrorKind::IndexOutOfBounds(index)))
    }

    pub fn get_bool_arg(&self, index: usize) -> EvalResult<bool> {
        match self.get_arg(index)? {
            Value::Bool(b) => Ok(*b),
            other => Err(Self::unexpected(other, "a boolean")),
        }
    }

    pub fn get_number_arg(&self, index: usize) -> EvalResult<&Number> {
        match self.get_arg(index)? {
            Value::Number(num) => Ok(num),
            other => Err(Self::unexpected(other, "a number")),
        }
    }

    pub fn get_str_arg(&self, index: usize) -> EvalResult<&str> {
        match self.get_arg(index)? {
            Value::String(string) => Ok(string),
            other => Err(Self::unexpected(other, "a string")),
        }
    }

    pub fn get_array_arg(&self, index: usize) -> EvalResult<&Vec<Value>> {
        match self.get_arg(index)? {
            Value::Array(array) => Ok(array),
            other => Err(Self::unexpected(other, "an array")),
        }
    }

    pub fn get_object_arg(&self, index: usize) -> EvalResult<&Map<String, Value>> {
        match self.get_arg(index)? {
            Value::Object(object) => Ok(object),
            other => Err(Self::unexpected(other, "an object")),
        }
    }

    fn unexpected(other: &Value, expected: &'static str) -> EvalError {
        EvalError::new(EvalErrorKind::Unexpected(other.clone().into(), expected))
    }
}
