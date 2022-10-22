use super::*;
use std::collections::VecDeque;

pub(super) fn evaluate_bool(expr: &Expression, ctx: &Context) -> EvalResult<bool> {
    match expr.evaluate(ctx)? {
        Value::Bool(value) => Ok(value),
        other => Err(ctx.error(Error::unexpected(other, "a boolean"))),
    }
}

pub(super) fn evaluate_string(expr: &Expression, ctx: &Context) -> EvalResult<String> {
    match expr.evaluate(ctx)? {
        Value::String(value) => Ok(value),
        other => Err(ctx.error(Error::unexpected(other, "a string"))),
    }
}

pub(super) fn evaluate_array(expr: &Expression, ctx: &Context) -> EvalResult<Vec<Value>> {
    match expr.evaluate(ctx)? {
        Value::Array(array) => Ok(array),
        other => Err(ctx.error(Error::unexpected(other, "an array"))),
    }
}

pub(super) fn evaluate_traversal(
    mut value: Value,
    mut operators: VecDeque<&TraversalOperator>,
    ctx: &Context,
) -> EvalResult<Value> {
    while let Some(operator) = operators.pop_front() {
        value = match operator {
            TraversalOperator::LegacyIndex(index) => {
                evaluate_array_value(value, *index as usize, ctx)?
            }
            TraversalOperator::Index(index_expr) => evaluate_index_expr(value, index_expr, ctx)?,
            TraversalOperator::GetAttr(name) => evaluate_object_value(value, name, ctx)?,
            TraversalOperator::AttrSplat => {
                // Consume all immediately following GetAttr operators and apply them to each array
                // element.
                let mut remaining = VecDeque::with_capacity(operators.len());

                while let Some(operator) = operators.pop_front() {
                    match operator {
                        TraversalOperator::GetAttr(_) => remaining.push_back(operator),
                        other => {
                            operators.push_front(other);
                            break;
                        }
                    }
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
) -> EvalResult<Value> {
    let array = match value {
        Value::Array(array) => array
            .into_iter()
            .map(|value| evaluate_traversal(value, operators.clone(), ctx))
            .collect::<EvalResult<_>>()?,
        Value::Null => vec![],
        other => evaluate_traversal(other, operators, ctx).map(|expr| vec![expr])?,
    };

    Ok(Value::Array(array))
}

fn evaluate_index_expr(value: Value, index_expr: &Expression, ctx: &Context) -> EvalResult<Value> {
    match index_expr.evaluate(ctx)? {
        Value::String(name) => evaluate_object_value(value, &name, ctx),
        Value::Number(num) => match num.as_u64() {
            Some(index) => evaluate_array_value(value, index as usize, ctx),
            None => Err(ctx.error(Error::unexpected(num, "an unsigned integer"))),
        },
        other => Err(ctx.error(Error::unexpected(other, "an unsigned integer or string"))),
    }
}

fn evaluate_array_value(mut value: Value, index: usize, ctx: &Context) -> EvalResult<Value> {
    match value.as_array_mut() {
        Some(array) => {
            if index < array.len() {
                Ok(array.swap_remove(index))
            } else {
                Err(ctx.error(ErrorKind::Index(index)))
            }
        }
        None => Err(ctx.error(Error::unexpected(value, "an array"))),
    }
}

fn evaluate_object_value(mut value: Value, key: &str, ctx: &Context) -> EvalResult<Value> {
    match value.as_object_mut() {
        Some(object) => object
            .swap_remove(key)
            .ok_or_else(|| ctx.error(ErrorKind::NoSuchKey(key.to_string()))),
        None => Err(ctx.error(Error::unexpected(value, "an object"))),
    }
}

fn evaluate_collection(expr: &Expression, ctx: &Context) -> EvalResult<Vec<(Value, Value)>> {
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
        other => Err(ctx.error(Error::unexpected(other, "an array or object"))),
    }
}

pub(super) struct Collection<'a> {
    ctx: &'a Context<'a>,
    key_var: Option<&'a Identifier>,
    value_var: &'a Identifier,
    cond_expr: Option<&'a Expression>,
    collection: Vec<(Value, Value)>,
}

impl<'a> Collection<'a> {
    pub(super) fn from_for_expr(for_expr: &'a ForExpr, ctx: &'a Context<'a>) -> EvalResult<Self> {
        Ok(Collection {
            ctx,
            key_var: for_expr.key_var.as_ref(),
            value_var: &for_expr.value_var,
            cond_expr: for_expr.cond_expr.as_ref(),
            collection: evaluate_collection(&for_expr.collection_expr, ctx)?,
        })
    }

    pub(super) fn from_for_directive(
        for_directive: &'a ForDirective,
        ctx: &'a Context<'a>,
    ) -> EvalResult<Self> {
        Ok(Collection {
            ctx,
            key_var: for_directive.key_var.as_ref(),
            value_var: &for_directive.value_var,
            cond_expr: None,
            collection: evaluate_collection(&for_directive.collection_expr, ctx)?,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.collection.len()
    }
}

impl<'a> IntoIterator for Collection<'a> {
    type Item = EvalResult<Context<'a>>;
    type IntoIter = IntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            ctx: self.ctx,
            key_var: self.key_var,
            value_var: self.value_var,
            cond_expr: self.cond_expr,
            iter: self.collection.into_iter(),
        }
    }
}

pub(super) struct IntoIter<'a> {
    ctx: &'a Context<'a>,
    key_var: Option<&'a Identifier>,
    value_var: &'a Identifier,
    cond_expr: Option<&'a Expression>,
    iter: std::vec::IntoIter<(Value, Value)>,
}

impl<'a> IntoIter<'a> {
    fn cond(&self, ctx: &Context) -> EvalResult<bool> {
        match &self.cond_expr {
            None => Ok(true),
            Some(cond_expr) => evaluate_bool(cond_expr, ctx),
        }
    }

    fn next_ctx(&mut self) -> Option<Context<'a>> {
        let (key, value) = self.iter.next()?;
        let mut ctx = self.ctx.child();
        if let Some(key_var) = self.key_var {
            ctx.declare_var(key_var.clone(), key);
        }

        ctx.declare_var(self.value_var.clone(), value);
        Some(ctx)
    }
}

impl<'a> Iterator for IntoIter<'a> {
    type Item = EvalResult<Context<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ctx = self.next_ctx()?;

            match self.cond(&ctx) {
                Ok(false) => {}
                Ok(true) => return Some(Ok(ctx)),
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
