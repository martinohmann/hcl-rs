use super::*;
use std::collections::VecDeque;

pub(super) fn evaluate_bool(expr: &Expression, ctx: &Context) -> EvalResult<bool> {
    match expr.evaluate(ctx)? {
        Expression::Bool(value) => Ok(value),
        other => Err(EvalError::unexpected(other, "a boolean")),
    }
}

pub(super) fn evaluate_array(expr: &Expression, ctx: &Context) -> EvalResult<Vec<Expression>> {
    match expr.evaluate(ctx)? {
        Expression::Array(array) => Ok(array),
        other => Err(EvalError::unexpected(other, "an array")),
    }
}

pub(super) fn evaluate_object(
    expr: &Expression,
    ctx: &Context,
) -> EvalResult<Object<ObjectKey, Expression>> {
    match expr.evaluate(ctx)? {
        Expression::Object(object) => Ok(object),
        other => Err(EvalError::unexpected(other, "an object")),
    }
}

pub(super) fn evaluate_collection(
    expr: &Expression,
    ctx: &Context,
) -> EvalResult<Object<ObjectKey, Expression>> {
    match expr.evaluate(ctx)? {
        Expression::Array(array) => Ok(array
            .into_iter()
            .enumerate()
            .map(|(index, value)| (ObjectKey::from(index), value))
            .collect()),
        Expression::Object(object) => Ok(object),
        other => Err(EvalError::unexpected(other, "an array or object")),
    }
}

pub(super) fn evaluate_traversal(
    expr: &Expression,
    mut operators: VecDeque<TraversalOperator>,
    ctx: &Context,
) -> EvalResult<Expression> {
    let mut expr = expr.clone();

    while let Some(operator) = operators.pop_front() {
        expr = match operator {
            TraversalOperator::LegacyIndex(index) => {
                evaluate_array_value(&expr, index as usize, ctx)?
            }
            TraversalOperator::Index(index_expr) => evaluate_index_expr(&expr, &index_expr, ctx)?,
            TraversalOperator::GetAttr(name) => {
                evaluate_object_value(&expr, name.into_inner(), ctx)?
            }
            TraversalOperator::AttrSplat => {
                // Consume all immediately following GetAttr operators and apply them to each array
                // element.
                let mut remaining = VecDeque::with_capacity(operators.len());

                while let Some(TraversalOperator::GetAttr(ident)) = operators.pop_front() {
                    remaining.push_back(TraversalOperator::GetAttr(ident));
                }

                let array = match expr.evaluate(ctx)? {
                    Expression::Array(array) => array
                        .iter()
                        .map(|expr| evaluate_traversal(expr, remaining.clone(), ctx))
                        .collect::<EvalResult<_>>()?,
                    Expression::Null => vec![],
                    other => evaluate_traversal(&other, remaining, ctx).map(|expr| vec![expr])?,
                };

                Expression::Array(array)
            }
            TraversalOperator::FullSplat => {
                // Consume all remaining operators and apply them to each array element.
                let remaining: VecDeque<TraversalOperator> = operators.drain(..).collect();

                let array = match expr.evaluate(ctx)? {
                    Expression::Array(array) => array
                        .iter()
                        .map(|expr| evaluate_traversal(expr, remaining.clone(), ctx))
                        .collect::<EvalResult<_>>()?,
                    Expression::Null => vec![],
                    other => evaluate_traversal(&other, remaining, ctx).map(|expr| vec![expr])?,
                };

                Expression::Array(array)
            }
        }
    }

    Ok(expr)
}

fn evaluate_index_expr(
    expr: &Expression,
    index_expr: &Expression,
    ctx: &Context,
) -> EvalResult<Expression> {
    match index_expr.evaluate(ctx)? {
        Expression::String(name) => evaluate_object_value(expr, name, ctx),
        Expression::Number(num) => match num.as_u64() {
            Some(index) => evaluate_array_value(expr, index as usize, ctx),
            None => Err(EvalError::unexpected(num, "an unsigned integer")),
        },
        other => Err(EvalError::unexpected(other, "a string or unsigned integer")),
    }
}

fn evaluate_array_value(expr: &Expression, index: usize, ctx: &Context) -> EvalResult<Expression> {
    let mut array = evaluate_array(expr, ctx)?;

    if index >= array.len() {
        return Err(EvalError::new(EvalErrorKind::IndexOutOfBounds(index)));
    }

    Ok(array.swap_remove(index))
}

fn evaluate_object_value(expr: &Expression, key: String, ctx: &Context) -> EvalResult<Expression> {
    let mut object = evaluate_object(expr, ctx)?;

    let key = ObjectKey::from(key);

    match object.swap_remove(&key) {
        Some(value) => Ok(value),
        None => Err(EvalError::new(EvalErrorKind::NoSuchKey(key))),
    }
}
