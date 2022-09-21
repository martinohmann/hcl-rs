use super::*;
use crate::{structure::*, template::Template, Number};

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
                eval_array_value(expr, index as usize, ctx)
            }
            ElementAccessOperator::Index(index) => eval_index_expr(expr, index.evaluate(ctx)?, ctx),
            ElementAccessOperator::GetAttr(name) => eval_object_value(expr, name.as_str(), ctx),
            ElementAccessOperator::AttrSplat => eval_attr_splat(expr, ctx),
            ElementAccessOperator::FullSplat => eval_full_splat(expr, ctx),
        }
    }
}

fn eval_index_expr(
    expr: Expression,
    index_expr: Expression,
    ctx: &Context,
) -> EvalResult<Expression> {
    match index_expr {
        Expression::String(name) => eval_object_value(expr, &name, ctx),
        Expression::Number(num) => match num.as_u64() {
            Some(index) => eval_array_value(expr, index as usize, ctx),
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

fn eval_array_value(expr: Expression, index: usize, ctx: &Context) -> EvalResult<Expression> {
    match expr {
        Expression::Array(mut array) => get_array_index(&mut array, index, ctx),
        other => Err(ctx.error(EvalErrorKind::Unexpected(other, "an array"))),
    }
}

fn eval_object_value(expr: Expression, key: &str, ctx: &Context) -> EvalResult<Expression> {
    match expr {
        Expression::Object(mut object) => get_object_value(&mut object, key, ctx),
        other => Err(ctx.error(EvalErrorKind::Unexpected(other, "an object"))),
    }
}

fn eval_attr_splat(expr: Expression, _ctx: &mut Context) -> EvalResult<Expression> {
    unimplemented!("evaluating attribute splat expression {expr} not implemented yet")
}

fn eval_full_splat(expr: Expression, _ctx: &mut Context) -> EvalResult<Expression> {
    unimplemented!("evaluating full splat expression {expr} not implemented yet")
}

fn get_array_index(
    array: &mut Vec<Expression>,
    index: usize,
    ctx: &Context,
) -> EvalResult<Expression> {
    if index >= array.len() {
        return Err(ctx.error(EvalErrorKind::IndexOutOfBounds(index)));
    }

    Ok(array.swap_remove(index))
}

fn get_object_value(
    object: &mut Object<ObjectKey, Expression>,
    key: &str,
    ctx: &Context,
) -> EvalResult<Expression> {
    let key = ObjectKey::from(key);

    match object.swap_remove(&key) {
        Some(value) => Ok(value),
        None => Err(ctx.error(EvalErrorKind::NoSuchKey(key.to_string()))),
    }
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
        match self.predicate.evaluate(ctx)? {
            Expression::Bool(cond) => {
                if cond {
                    self.true_expr.evaluate(ctx)
                } else {
                    self.false_expr.evaluate(ctx)
                }
            }
            other => Err(ctx.error(EvalErrorKind::Unexpected(other, "a boolean"))),
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
            (operator, expr) => Err(ctx.error(EvalErrorKind::Message(format!(
                "operator `{}` cannot be applied to expression `{}`",
                operator.as_str(),
                expr
            )))),
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
            (Number(lhs), LessEq, Number(rhs)) => eval_numbers(lhs, rhs, |a, b| a <= b),
            (Number(lhs), GreaterEq, Number(rhs)) => eval_numbers(lhs, rhs, |a, b| a >= b),
            (Number(lhs), Less, Number(rhs)) => eval_numbers(lhs, rhs, |a, b| a < b),
            (Number(lhs), Greater, Number(rhs)) => eval_numbers(lhs, rhs, |a, b| a > b),
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

    fn evaluate(self, _ctx: &mut Context) -> EvalResult<Self::Output> {
        todo!()
    }
}
