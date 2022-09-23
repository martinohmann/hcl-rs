use super::*;
use crate::{ElementAccessOperator, Object};
use std::collections::VecDeque;

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

pub(super) fn evaluate_element_access(
    mut expr: Expression,
    mut operators: VecDeque<ElementAccessOperator>,
    ctx: &Context,
) -> EvalResult<Expression> {
    while let Some(operator) = operators.pop_front() {
        expr = match operator {
            ElementAccessOperator::LegacyIndex(index) => {
                evaluate_array_value(expr, index as usize, ctx)?
            }
            ElementAccessOperator::Index(index_expr) => evaluate_index_expr(expr, index_expr, ctx)?,
            ElementAccessOperator::GetAttr(name) => {
                evaluate_object_value(expr, name.into_inner(), ctx)?
            }
            ElementAccessOperator::AttrSplat => {
                // Consume all immediately following GetAttr operators and apply them to each array
                // element.
                let mut remaining = VecDeque::with_capacity(operators.len());

                while let Some(ElementAccessOperator::GetAttr(ident)) = operators.pop_front() {
                    remaining.push_back(ElementAccessOperator::GetAttr(ident));
                }

                let array = match expr.evaluate(ctx)? {
                    Expression::Array(array) => array
                        .into_iter()
                        .map(|expr| evaluate_element_access(expr, remaining.clone(), ctx))
                        .collect::<EvalResult<_>>()?,
                    Expression::Null => vec![],
                    other => {
                        evaluate_element_access(other, remaining, ctx).map(|expr| vec![expr])?
                    }
                };

                Expression::Array(array)
            }
            ElementAccessOperator::FullSplat => {
                // Consume all remaining access operators and apply them to each array element.
                let remaining: VecDeque<ElementAccessOperator> = operators.drain(..).collect();

                let array = match expr.evaluate(ctx)? {
                    Expression::Array(array) => array
                        .into_iter()
                        .map(|expr| evaluate_element_access(expr, remaining.clone(), ctx))
                        .collect::<EvalResult<_>>()?,
                    Expression::Null => vec![],
                    other => {
                        evaluate_element_access(other, remaining, ctx).map(|expr| vec![expr])?
                    }
                };

                Expression::Array(array)
            }
        }
    }

    Ok(expr)
}

fn evaluate_index_expr(
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

fn evaluate_array_value(expr: Expression, index: usize, ctx: &Context) -> EvalResult<Expression> {
    let mut array = evaluate_array(expr, ctx)?;

    if index >= array.len() {
        return Err(ctx.error(EvalErrorKind::IndexOutOfBounds(index)));
    }

    Ok(array.swap_remove(index))
}

fn evaluate_object_value(expr: Expression, key: String, ctx: &Context) -> EvalResult<Expression> {
    let mut object = evaluate_object(expr, ctx)?;

    let key = ObjectKey::from(key);

    match object.swap_remove(&key) {
        Some(value) => Ok(value),
        None => Err(ctx.error(EvalErrorKind::NoSuchKey(key.to_string()))),
    }
}
