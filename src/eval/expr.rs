use super::*;
use crate::Object;

pub(super) fn evaluate_bool(expr: Expression, ctx: &Context) -> EvalResult<bool> {
    match expr.evaluate(ctx)? {
        Expression::Bool(value) => Ok(value),
        other => Err(ctx.error(EvalErrorKind::Unexpected(other, "a boolean"))),
    }
}

pub(super) fn evaluate_string(expr: Expression, ctx: &Context) -> EvalResult<String> {
    match expr.evaluate(ctx)? {
        Expression::String(value) => Ok(value),
        other => Err(ctx.error(EvalErrorKind::Unexpected(other, "a string"))),
    }
}

pub(super) fn evaluate_array(expr: Expression, ctx: &Context) -> EvalResult<Vec<Expression>> {
    match expr.evaluate(ctx)? {
        Expression::Array(array) => Ok(array),
        other => Err(ctx.error(EvalErrorKind::Unexpected(other, "an array"))),
    }
}

pub(super) fn evaluate_object(
    expr: Expression,
    ctx: &Context,
) -> EvalResult<Object<ObjectKey, Expression>> {
    match expr.evaluate(ctx)? {
        Expression::Object(object) => Ok(object),
        other => Err(ctx.error(EvalErrorKind::Unexpected(other, "an object"))),
    }
}

pub(super) fn evaluate_index_expr(
    expr: Expression,
    index_expr: Expression,
    ctx: &Context,
) -> EvalResult<Expression> {
    match index_expr.evaluate(ctx)? {
        Expression::String(name) => evaluate_object_value(expr, name, ctx),
        Expression::Number(num) => match num.as_u64() {
            Some(index) => evaluate_array_value(expr, index as usize, ctx),
            None => Err(ctx.error(EvalErrorKind::Unexpected(
                Expression::Number(num),
                "an unsigned integer",
            ))),
        },
        other => Err(ctx.error(EvalErrorKind::Unexpected(
            other,
            "a string or unsigned integer",
        ))),
    }
}

pub(super) fn evaluate_array_value(
    expr: Expression,
    index: usize,
    ctx: &Context,
) -> EvalResult<Expression> {
    let mut array = expr::evaluate_array(expr, ctx)?;

    if index >= array.len() {
        return Err(ctx.error(EvalErrorKind::IndexOutOfBounds(index)));
    }

    Ok(array.swap_remove(index))
}

pub(super) fn evaluate_object_value(
    expr: Expression,
    key: String,
    ctx: &Context,
) -> EvalResult<Expression> {
    let mut object = expr::evaluate_object(expr, ctx)?;

    let key = ObjectKey::from(key);

    match object.swap_remove(&key) {
        Some(value) => Ok(value),
        None => Err(ctx.error(EvalErrorKind::NoSuchKey(key.to_string()))),
    }
}

pub(super) fn evaluate_attr_splat(expr: Expression, _ctx: &Context) -> EvalResult<Expression> {
    unimplemented!("evaluating attribute splat expression {expr} not implemented yet")
}

pub(super) fn evaluate_full_splat(expr: Expression, _ctx: &Context) -> EvalResult<Expression> {
    unimplemented!("evaluating full splat expression {expr} not implemented yet")
}
