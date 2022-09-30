use super::*;
use std::collections::VecDeque;
use vecmap::VecMap;

pub(super) fn evaluate_bool(expr: &Expression, ctx: &Context) -> EvalResult<bool> {
    match expr.evaluate(ctx)? {
        Value::Bool(value) => Ok(value),
        other => Err(EvalError::unexpected(other, "a boolean")),
    }
}

pub(super) fn evaluate_string(expr: &Expression, ctx: &Context) -> EvalResult<String> {
    match expr.evaluate(ctx)? {
        Value::String(value) => Ok(value),
        other => Err(EvalError::unexpected(other, "a string")),
    }
}

pub(super) fn evaluate_array(expr: &Expression, ctx: &Context) -> EvalResult<Vec<Value>> {
    match expr.evaluate(ctx)? {
        Value::Array(array) => Ok(array),
        other => Err(EvalError::unexpected(other, "an array")),
    }
}

pub(super) fn evaluate_collection(
    expr: &Expression,
    ctx: &Context,
) -> EvalResult<VecMap<Value, Value>> {
    match expr.evaluate(ctx)? {
        Value::Array(array) => Ok(array
            .into_iter()
            .enumerate()
            .map(|(index, value)| (Value::from(index), value))
            .collect()),
        Value::Object(object) => Ok(object
            .into_iter()
            .map(|(key, value)| (Value::from(key), value))
            .collect()),
        other => Err(EvalError::unexpected(other, "an array or object")),
    }
}

pub(super) fn evaluate_traversal(
    mut value: Value,
    mut operators: VecDeque<TraversalOperator>,
    ctx: &Context,
) -> EvalResult<Value> {
    while let Some(operator) = operators.pop_front() {
        value = match operator {
            TraversalOperator::LegacyIndex(index) => evaluate_array_value(value, index as usize)?,
            TraversalOperator::Index(index_expr) => evaluate_index_expr(value, &index_expr, ctx)?,
            TraversalOperator::GetAttr(name) => evaluate_object_value(value, name.into_inner())?,
            TraversalOperator::AttrSplat => {
                // Consume all immediately following GetAttr operators and apply them to each array
                // element.
                let mut remaining = VecDeque::with_capacity(operators.len());

                while let Some(TraversalOperator::GetAttr(ident)) = operators.pop_front() {
                    remaining.push_back(TraversalOperator::GetAttr(ident));
                }

                let array = match value {
                    Value::Array(array) => array
                        .into_iter()
                        .map(|value| evaluate_traversal(value, remaining.clone(), ctx))
                        .collect::<EvalResult<_>>()?,
                    Value::Null => vec![],
                    other => evaluate_traversal(other, remaining, ctx).map(|expr| vec![expr])?,
                };

                Value::Array(array)
            }
            TraversalOperator::FullSplat => {
                // Consume all remaining operators and apply them to each array element.
                let remaining: VecDeque<TraversalOperator> = operators.drain(..).collect();

                let array = match value {
                    Value::Array(array) => array
                        .into_iter()
                        .map(|value| evaluate_traversal(value, remaining.clone(), ctx))
                        .collect::<EvalResult<_>>()?,
                    Value::Null => vec![],
                    other => evaluate_traversal(other, remaining, ctx).map(|expr| vec![expr])?,
                };

                Value::Array(array)
            }
        }
    }

    Ok(value)
}

fn evaluate_index_expr(value: Value, index_expr: &Expression, ctx: &Context) -> EvalResult<Value> {
    match index_expr.evaluate(ctx)? {
        Value::String(name) => evaluate_object_value(value, name),
        Value::Number(num) => match num.as_u64() {
            Some(index) => evaluate_array_value(value, index as usize),
            None => Err(EvalError::unexpected(num, "an unsigned integer")),
        },
        other => Err(EvalError::unexpected(other, "a string or unsigned integer")),
    }
}

fn evaluate_array_value(mut value: Value, index: usize) -> EvalResult<Value> {
    match value.as_array_mut() {
        Some(array) => {
            if index >= array.len() {
                return Err(EvalError::new(EvalErrorKind::IndexOutOfBounds(index)));
            }

            Ok(array.swap_remove(index))
        }
        None => Err(EvalError::unexpected(value, "an array")),
    }
}

fn evaluate_object_value(mut value: Value, key: String) -> EvalResult<Value> {
    match value.as_object_mut() {
        Some(object) => match object.swap_remove(&key) {
            Some(value) => Ok(value),
            None => Err(EvalError::new(EvalErrorKind::NoSuchKey(key))),
        },
        None => Err(EvalError::unexpected(value, "an object")),
    }
}
