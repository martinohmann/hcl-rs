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
        if self.args.len() != count {
            Err(EvalError::new(EvalErrorKind::Message(format!(
                "unexpected argument count, expected {} but got {}",
                count,
                self.args.len()
            ))))
        } else {
            Ok(())
        }
    }

    pub fn ensure_min_args(&self, count: usize) -> EvalResult<()> {
        if self.args.len() < count {
            Err(EvalError::new(EvalErrorKind::Message(format!(
                "unexpected argument count, expected at least {} but got {}",
                count,
                self.args.len()
            ))))
        } else {
            Ok(())
        }
    }

    pub fn get_arg(&self, index: usize) -> EvalResult<&Value> {
        self.args
            .get(index)
            .ok_or_else(|| EvalError::new(EvalErrorKind::IndexOutOfBounds(index)))
    }

    pub fn get_number_arg(&self, index: usize) -> EvalResult<&Number> {
        match self.get_arg(index)? {
            Value::Number(num) => Ok(num),
            other => Err(EvalError::new(EvalErrorKind::Unexpected(
                other.clone().into(),
                "a number",
            ))),
        }
    }

    pub fn args(&self) -> &[Value] {
        &self.args
    }
}
