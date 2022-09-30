use super::*;
use vecmap::VecMap;

pub(super) struct Collection<'a> {
    ctx: &'a Context<'a>,
    key_var: Option<&'a Identifier>,
    value_var: &'a Identifier,
    cond_expr: Option<&'a Expression>,
    collection: VecMap<Value, Value>,
}

impl<'a> Collection<'a> {
    pub(super) fn from_for_expr(for_expr: &'a ForExpr, ctx: &'a Context<'a>) -> EvalResult<Self> {
        Ok(Collection {
            ctx,
            key_var: for_expr.key_var.as_ref(),
            value_var: &for_expr.value_var,
            cond_expr: for_expr.cond_expr.as_ref(),
            collection: expr::evaluate_collection(&for_expr.collection_expr, ctx)?,
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
            collection: expr::evaluate_collection(&for_directive.collection_expr, ctx)?,
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
    iter: vecmap::map::IntoIter<Value, Value>,
}

impl<'a> IntoIter<'a> {
    fn cond(&self, ctx: &Context) -> EvalResult<bool> {
        match &self.cond_expr {
            None => Ok(true),
            Some(cond_expr) => expr::evaluate_bool(cond_expr, ctx),
        }
    }

    fn next_ctx(&mut self) -> Option<Context<'a>> {
        let (key, value) = self.iter.next()?;
        let mut ctx = self.ctx.new_child();
        if let Some(key_var) = self.key_var {
            ctx.set_var(key_var.clone(), key);
        }

        ctx.set_var(self.value_var.clone(), value);
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
