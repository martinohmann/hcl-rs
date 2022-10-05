use super::*;
use std::collections::VecDeque;

pub(super) fn evaluate_bool(expr: &Expression, ctx: &Context) -> Result<bool> {
    match expr.evaluate(ctx)? {
        Value::Bool(value) => Ok(value),
        other => Err(Error::unexpected(other, "a boolean")),
    }
}

pub(super) fn evaluate_string(expr: &Expression, ctx: &Context) -> Result<String> {
    match expr.evaluate(ctx)? {
        Value::String(value) => Ok(value),
        other => Err(Error::unexpected(other, "a string")),
    }
}

pub(super) fn evaluate_array(expr: &Expression, ctx: &Context) -> Result<Vec<Value>> {
    match expr.evaluate(ctx)? {
        Value::Array(array) => Ok(array),
        other => Err(Error::unexpected(other, "an array")),
    }
}

pub(super) fn evaluate_collection(expr: &Expression, ctx: &Context) -> Result<Vec<(Value, Value)>> {
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
        other => Err(Error::unexpected(other, "an array or object")),
    }
}

pub(super) fn evaluate_traversal(
    mut value: Value,
    mut operators: VecDeque<&TraversalOperator>,
    ctx: &Context,
) -> Result<Value> {
    while let Some(operator) = operators.pop_front() {
        value = match operator {
            TraversalOperator::LegacyIndex(index) => evaluate_array_value(value, *index as usize)?,
            TraversalOperator::Index(index_expr) => evaluate_index_expr(value, index_expr, ctx)?,
            TraversalOperator::GetAttr(name) => evaluate_object_value(value, name)?,
            TraversalOperator::AttrSplat => {
                // Consume all immediately following GetAttr operators and apply them to each array
                // element.
                let mut remaining = VecDeque::with_capacity(operators.len());

                while let Some(op @ TraversalOperator::GetAttr(_)) = operators.pop_front() {
                    remaining.push_back(op);
                }

                evaluate_splat(value, remaining, ctx)?
            }
            TraversalOperator::FullSplat => {
                // Consume all remaining operators and apply them to each array element.
                let remaining: VecDeque<&TraversalOperator> = operators.drain(..).collect();

                evaluate_splat(value, remaining, ctx)?
            }
        }
    }

    Ok(value)
}

fn evaluate_splat(
    value: Value,
    operators: VecDeque<&TraversalOperator>,
    ctx: &Context,
) -> Result<Value> {
    let array = match value {
        Value::Array(array) => array
            .into_iter()
            .map(|value| evaluate_traversal(value, operators.clone(), ctx))
            .collect::<Result<_>>()?,
        Value::Null => vec![],
        other => evaluate_traversal(other, operators, ctx).map(|expr| vec![expr])?,
    };

    Ok(Value::Array(array))
}

fn evaluate_index_expr(value: Value, index_expr: &Expression, ctx: &Context) -> Result<Value> {
    match index_expr.evaluate(ctx)? {
        Value::String(name) => evaluate_object_value(value, &name),
        Value::Number(num) => match num.as_u64() {
            Some(index) => evaluate_array_value(value, index as usize),
            None => Err(Error::unexpected(num, "an unsigned integer")),
        },
        other => Err(Error::unexpected(other, "an unsigned integer or string")),
    }
}

fn evaluate_array_value(mut value: Value, index: usize) -> Result<Value> {
    match value.as_array_mut() {
        Some(array) => {
            if index < array.len() {
                Ok(array.swap_remove(index))
            } else {
                Err(Error::new(ErrorKind::IndexOutOfBounds(index)))
            }
        }
        None => Err(Error::unexpected(value, "an array")),
    }
}

fn evaluate_object_value(mut value: Value, key: &str) -> Result<Value> {
    match value.as_object_mut() {
        Some(object) => object
            .swap_remove(key)
            .ok_or_else(|| Error::new(ErrorKind::NoSuchKey(key.to_string()))),
        None => Err(Error::unexpected(value, "an object")),
    }
}
