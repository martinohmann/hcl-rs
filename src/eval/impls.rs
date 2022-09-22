use super::{expr::EvaluateExpr, *};
use crate::{structure::*, template::Template, Number};
use vecmap::map::Entry;

impl private::Sealed for Body {}

impl Evaluate for Body {
    type Output = Self;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        self.into_iter()
            .map(|structure| structure.evaluate(ctx))
            .collect::<EvalResult<Body>>()
    }
}

impl private::Sealed for Structure {}

impl Evaluate for Structure {
    type Output = Self;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        match self {
            Structure::Attribute(attr) => attr.evaluate(ctx).map(Structure::Attribute),
            Structure::Block(block) => block.evaluate(ctx).map(Structure::Block),
        }
    }
}

impl private::Sealed for Attribute {}

impl Evaluate for Attribute {
    type Output = Self;

    fn evaluate(mut self, ctx: &mut Context) -> EvalResult<Self::Output> {
        let mut ctx = ctx.new_scope(Scope::Attr(&self.key));
        self.expr = self.expr.evaluate(&mut ctx)?;
        Ok(self)
    }
}

impl private::Sealed for Block {}

impl Evaluate for Block {
    type Output = Self;

    fn evaluate(mut self, ctx: &mut Context) -> EvalResult<Self::Output> {
        let mut ctx = ctx.new_scope(Scope::Block(&self.identifier, &self.labels));
        self.body = self.body.evaluate(&mut ctx)?;
        Ok(self)
    }
}

impl private::Sealed for Expression {}

impl Evaluate for Expression {
    type Output = Self;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        let scope_expr = self.clone();
        let mut ctx = ctx.new_scope(Scope::Expr(&scope_expr));

        match self {
            Expression::Array(array) => array.evaluate(&mut ctx).map(Expression::Array),
            Expression::Object(object) => object.evaluate(&mut ctx).map(Expression::Object),
            Expression::TemplateExpr(expr) => expr.evaluate(&mut ctx).map(Expression::String),
            Expression::VariableExpr(ident) => {
                ctx.get_variable(ident.as_str()).cloned().map(Into::into)
            }
            Expression::ElementAccess(access) => access.evaluate(&mut ctx),
            Expression::FuncCall(func_call) => func_call.evaluate(&mut ctx),
            Expression::SubExpr(expr) => expr.evaluate(&mut ctx),
            Expression::Conditional(cond) => cond.evaluate(&mut ctx),
            Expression::Operation(op) => op.evaluate(&mut ctx),
            Expression::ForExpr(expr) => expr.evaluate(&mut ctx),
            Expression::Raw(_) => Err(ctx.error(EvalErrorKind::RawExpression)),
            other => Ok(other),
        }
    }
}

impl private::Sealed for Vec<Expression> {}

impl Evaluate for Vec<Expression> {
    type Output = Self;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        self.into_iter()
            .enumerate()
            .map(|(index, expr)| {
                let mut ctx = ctx.new_scope(Scope::Index(index));
                expr.evaluate(&mut ctx)
            })
            .collect()
    }
}

impl private::Sealed for Object<ObjectKey, Expression> {}

impl Evaluate for Object<ObjectKey, Expression> {
    type Output = Self;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        self.into_iter()
            .map(|(key, expr)| {
                let scope_key = key.clone();
                let mut ctx = ctx.new_scope(Scope::Key(&scope_key));
                Ok((key.evaluate(&mut ctx)?, expr.evaluate(&mut ctx)?))
            })
            .collect()
    }
}

impl private::Sealed for ObjectKey {}

impl Evaluate for ObjectKey {
    type Output = Self;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        match self {
            ObjectKey::Expression(expr) => expr.evaluate(ctx).map(ObjectKey::Expression),
            ident => Ok(ident),
        }
    }
}

impl private::Sealed for TemplateExpr {}

impl Evaluate for TemplateExpr {
    type Output = String;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        match Template::from_expr(&self) {
            Ok(template) => template.evaluate(ctx),
            Err(err) => Err(err.into()),
        }
    }
}

impl private::Sealed for Template {}

impl Evaluate for Template {
    type Output = String;

    fn evaluate(self, _ctx: &mut Context) -> EvalResult<Self::Output> {
        todo!()
    }
}

impl private::Sealed for ElementAccess {}

impl Evaluate for ElementAccess {
    type Output = Expression;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        let expr = self.expr.evaluate(ctx)?;

        match self.operator {
            ElementAccessOperator::LegacyIndex(index) => {
                evaluate_array_value(expr, index as usize, ctx)
            }
            ElementAccessOperator::Index(index_expr) => evaluate_index_expr(expr, index_expr, ctx),
            ElementAccessOperator::GetAttr(name) => {
                evaluate_object_value(expr, name.into_inner(), ctx)
            }
            ElementAccessOperator::AttrSplat => evaluate_attr_splat(expr, ctx),
            ElementAccessOperator::FullSplat => evaluate_full_splat(expr, ctx),
        }
    }
}

fn evaluate_index_expr(
    expr: Expression,
    index_expr: Expression,
    ctx: &mut Context,
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

fn evaluate_array_value(
    expr: Expression,
    index: usize,
    ctx: &mut Context,
) -> EvalResult<Expression> {
    let mut array = expr.evaluate_array(ctx)?;

    if index >= array.len() {
        return Err(ctx.error(EvalErrorKind::IndexOutOfBounds(index)));
    }

    Ok(array.swap_remove(index))
}

fn evaluate_object_value(
    expr: Expression,
    key: String,
    ctx: &mut Context,
) -> EvalResult<Expression> {
    let mut object = expr.evaluate_object(ctx)?;

    let key = ObjectKey::from(key);

    match object.swap_remove(&key) {
        Some(value) => Ok(value),
        None => Err(ctx.error(EvalErrorKind::NoSuchKey(key.to_string()))),
    }
}

fn evaluate_attr_splat(expr: Expression, _ctx: &mut Context) -> EvalResult<Expression> {
    unimplemented!("evaluating attribute splat expression {expr} not implemented yet")
}

fn evaluate_full_splat(expr: Expression, _ctx: &mut Context) -> EvalResult<Expression> {
    unimplemented!("evaluating full splat expression {expr} not implemented yet")
}

impl private::Sealed for FuncCall {}

impl Evaluate for FuncCall {
    type Output = Expression;

    fn evaluate(self, _ctx: &mut Context) -> EvalResult<Self::Output> {
        todo!()
    }
}

impl private::Sealed for Conditional {}

impl Evaluate for Conditional {
    type Output = Expression;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        if self.predicate.evaluate_bool(ctx)? {
            self.true_expr.evaluate(ctx)
        } else {
            self.false_expr.evaluate(ctx)
        }
    }
}

impl private::Sealed for Operation {}

impl Evaluate for Operation {
    type Output = Expression;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        match self {
            Operation::Unary(unary) => unary.evaluate(ctx),
            Operation::Binary(binary) => binary.evaluate(ctx),
        }
    }
}

impl private::Sealed for UnaryOp {}

impl Evaluate for UnaryOp {
    type Output = Expression;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        let expr = self.expr.evaluate(ctx)?;

        match (self.operator, expr) {
            (UnaryOperator::Not, Expression::Bool(v)) => Ok(Expression::Bool(!v)),
            (UnaryOperator::Neg, Expression::Number(n)) => Ok(Expression::Number(-n)),
            (operator, expr) => Err(ctx.error(EvalErrorKind::InvalidUnaryOp(operator, expr))),
        }
    }
}

impl private::Sealed for BinaryOp {}

impl Evaluate for BinaryOp {
    type Output = Expression;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        use {BinaryOperator::*, Expression::*};

        let op = self.normalize();
        let lhs = op.lhs_expr.evaluate(ctx)?;
        let rhs = op.rhs_expr.evaluate(ctx)?;

        let expr = match (lhs, op.operator, rhs) {
            (lhs, Eq, rhs) => Bool(lhs == rhs),
            (lhs, NotEq, rhs) => Bool(lhs != rhs),
            (Bool(lhs), And, Bool(rhs)) => Bool(lhs && rhs),
            (Bool(lhs), Or, Bool(rhs)) => Bool(lhs || rhs),
            (Number(lhs), LessEq, Number(rhs)) => Bool(lhs <= rhs),
            (Number(lhs), GreaterEq, Number(rhs)) => Bool(lhs >= rhs),
            (Number(lhs), Less, Number(rhs)) => Bool(lhs < rhs),
            (Number(lhs), Greater, Number(rhs)) => Bool(lhs > rhs),
            (Number(lhs), Plus, Number(rhs)) => eval_numbers(lhs, rhs, |a, b| a + b),
            (Number(lhs), Minus, Number(rhs)) => eval_numbers(lhs, rhs, |a, b| a - b),
            (Number(lhs), Mul, Number(rhs)) => eval_numbers(lhs, rhs, |a, b| a * b),
            (Number(lhs), Div, Number(rhs)) => eval_numbers(lhs, rhs, |a, b| a / b),
            (Number(lhs), Mod, Number(rhs)) => eval_numbers(lhs, rhs, |a, b| a % b),
            (lhs, operator, rhs) => {
                return Err(ctx.error(EvalErrorKind::InvalidBinaryOp(lhs, operator, rhs)))
            }
        };

        Ok(expr)
    }
}

fn eval_numbers<F, T>(lhs: Number, rhs: Number, f: F) -> Expression
where
    F: FnOnce(f64, f64) -> T,
    T: Into<Expression>,
{
    f(lhs.as_f64().unwrap(), rhs.as_f64().unwrap()).into()
}

impl private::Sealed for ForExpr {}

impl Evaluate for ForExpr {
    type Output = Expression;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        match self {
            ForExpr::List(expr) => expr.evaluate(ctx).map(Expression::Array),
            ForExpr::Object(expr) => expr.evaluate(ctx).map(Expression::Object),
        }
    }
}

impl private::Sealed for ForListExpr {}

impl Evaluate for ForListExpr {
    type Output = Vec<Expression>;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        let values = self.intro.expr.evaluate_array(ctx)?;
        let key_var = self.intro.key.as_ref().map(|key| key.as_str());
        let value_var = self.intro.value.as_str();

        let mut result = Vec::with_capacity(values.len());

        for (index, value) in values.into_iter().enumerate() {
            let mut ctx = ctx.new_scope(Scope::Index(index));
            if let Some(key_var) = &key_var {
                ctx.set_variable(key_var.to_string(), index);
            }

            ctx.set_variable(value_var.to_owned(), value);

            let keep = match &self.cond {
                None => true,
                Some(cond) => cond.clone().evaluate_bool(&mut ctx)?,
            };

            if keep {
                result.push(self.expr.clone().evaluate(&mut ctx)?);
            }
        }

        Ok(result)
    }
}

impl private::Sealed for ForObjectExpr {}

impl Evaluate for ForObjectExpr {
    type Output = Object<ObjectKey, Expression>;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        let object = self.intro.expr.evaluate_object(ctx)?;
        let key_var = self.intro.key.as_ref().map(|key| key.as_str());
        let value_var = self.intro.value.as_str();

        fn keep(cond: Option<&Expression>, ctx: &mut Context) -> EvalResult<bool> {
            match cond {
                Some(cond) => cond.clone().evaluate_bool(ctx),
                None => Ok(true),
            }
        }

        if self.value_grouping {
            let mut result: Object<String, Vec<Expression>> = Object::with_capacity(object.len());

            for (key, value) in object.into_iter() {
                let mut ctx = ctx.new_scope(Scope::Key(&key));
                if let Some(key_var) = &key_var {
                    ctx.set_variable(key_var.to_string(), key.to_string());
                }

                ctx.set_variable(value_var.to_string(), value);

                if keep(self.cond.as_ref(), &mut ctx)? {
                    let key = self.key_expr.clone().evaluate_string(&mut ctx)?;
                    let value = self.value_expr.clone().evaluate(&mut ctx)?;

                    result.entry(key).or_default().push(value);
                }
            }

            Ok(result
                .into_iter()
                .map(|(k, v)| (ObjectKey::from(k), Expression::Array(v)))
                .collect())
        } else {
            let mut result: Object<String, Expression> = Object::with_capacity(object.len());

            for (key, value) in object.into_iter() {
                let mut ctx = ctx.new_scope(Scope::Key(&key));
                if let Some(key_var) = &key_var {
                    ctx.set_variable(key_var.to_string(), key.to_string());
                }

                ctx.set_variable(value_var.to_string(), value);

                if keep(self.cond.as_ref(), &mut ctx)? {
                    let key = self.key_expr.clone().evaluate_string(&mut ctx)?;

                    match result.entry(key) {
                        Entry::Occupied(entry) => {
                            return Err(ctx.error(EvalErrorKind::KeyAlreadyExists(entry.into_key())))
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(self.value_expr.clone().evaluate(&mut ctx)?);
                        }
                    }
                }
            }

            Ok(result
                .into_iter()
                .map(|(k, v)| (ObjectKey::from(k), v))
                .collect())
        }
    }
}
