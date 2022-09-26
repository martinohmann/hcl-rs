use super::*;
use crate::{ForExpr, Object};

pub(super) struct Collection<'a> {
    ctx: &'a Context<'a>,
    for_expr: &'a ForExpr,
    collection: Object<ObjectKey, Expression>,
}

impl<'a> Collection<'a> {
    pub(super) fn new(for_expr: &'a ForExpr, ctx: &'a Context<'a>) -> EvalResult<Self> {
        Ok(Collection {
            ctx,
            for_expr,
            collection: expr::evaluate_collection(for_expr.collection_expr.clone(), ctx)?,
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
            for_expr: self.for_expr,
            iter: self.collection.into_iter(),
        }
    }
}

pub struct IntoIter<'a> {
    ctx: &'a Context<'a>,
    for_expr: &'a ForExpr,
    iter: vecmap::map::IntoIter<ObjectKey, Expression>,
}

impl<'a> IntoIter<'a> {
    fn cond(&self, ctx: &Context) -> EvalResult<bool> {
        match &self.for_expr.cond_expr {
            None => Ok(true),
            Some(cond_expr) => expr::evaluate_bool(cond_expr.clone(), ctx),
        }
    }

    fn iteration_ctx(&self, key: ObjectKey, value: Expression) -> Context<'a> {
        let mut ctx = self.ctx.new_scope();
        if let Some(key_var) = &self.for_expr.key_var {
            ctx.set_variable(key_var.as_str().to_string(), key);
        }

        ctx.set_variable(self.for_expr.value_var.as_str().to_string(), value);
        ctx
    }
}

impl<'a> Iterator for IntoIter<'a> {
    type Item = EvalResult<Context<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (key, value) = self.iter.next()?;
            let ctx = self.iteration_ctx(key, value);

            match self.cond(&ctx) {
                Ok(false) => {}
                Ok(true) => return Some(Ok(ctx)),
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
