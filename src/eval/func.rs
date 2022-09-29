use super::*;
use crate::Number;

pub type Func = fn(FuncArgs) -> EvalResult<Value>;

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
