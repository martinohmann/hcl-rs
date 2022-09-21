use super::*;
use crate::Object;

pub(super) trait EvaluateExpr: Evaluate {
    fn evaluate_bool(self, ctx: &mut Context) -> EvalResult<bool>;

    fn evaluate_string(self, ctx: &mut Context) -> EvalResult<String>;

    fn evaluate_array(self, ctx: &mut Context) -> EvalResult<Vec<Expression>>;

    fn evaluate_object(self, ctx: &mut Context) -> EvalResult<Object<ObjectKey, Expression>>;
}

impl EvaluateExpr for Expression {
    fn evaluate_bool(self, ctx: &mut Context) -> EvalResult<bool> {
        match self.evaluate(ctx)? {
            Expression::Bool(value) => Ok(value),
            other => Err(ctx.error(EvalErrorKind::Unexpected(other, "a boolean"))),
        }
    }

    fn evaluate_string(self, ctx: &mut Context) -> EvalResult<String> {
        match self.evaluate(ctx)? {
            Expression::String(value) => Ok(value),
            other => Err(ctx.error(EvalErrorKind::Unexpected(other, "a string"))),
        }
    }

    fn evaluate_array(self, ctx: &mut Context) -> EvalResult<Vec<Expression>> {
        match self.evaluate(ctx)? {
            Expression::Array(array) => Ok(array),
            other => Err(ctx.error(EvalErrorKind::Unexpected(other, "an array"))),
        }
    }

    fn evaluate_object(self, ctx: &mut Context) -> EvalResult<Object<ObjectKey, Expression>> {
        match self.evaluate(ctx)? {
            Expression::Object(object) => Ok(object),
            other => Err(ctx.error(EvalErrorKind::Unexpected(other, "an object"))),
        }
    }
}
