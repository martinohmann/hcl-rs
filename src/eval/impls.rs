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
        self.expr = self.expr.evaluate(ctx)?;
        Ok(self)
    }
}

impl private::Sealed for Block {}

impl Evaluate for Block {
    type Output = Self;

    fn evaluate(mut self, ctx: &mut Context) -> EvalResult<Self::Output> {
        self.body = self.body.evaluate(ctx)?;
        Ok(self)
    }
}

impl private::Sealed for Expression {}

impl Evaluate for Expression {
    type Output = Self;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        match self {
            Expression::Array(array) => array.evaluate(ctx).map(Expression::Array),
            Expression::Object(object) => object.evaluate(ctx).map(Expression::Object),
            Expression::TemplateExpr(expr) => expr.evaluate(ctx).map(Expression::String),
            Expression::VariableExpr(ident) => {
                ctx.get_variable(ident.as_str()).cloned().map(Into::into)
            }
            Expression::ElementAccess(access) => access.evaluate(ctx),
            Expression::FuncCall(func_call) => func_call.evaluate(ctx),
            Expression::SubExpr(expr) => expr.evaluate(ctx),
            Expression::Conditional(cond) => cond.evaluate(ctx),
            Expression::Operation(op) => op.evaluate(ctx),
            Expression::ForExpr(expr) => expr.evaluate(ctx),
            Expression::Raw(_) => Err(EvalError::from("raw expressions cannot be evaluated")),
            other => Ok(other),
        }
    }
}

impl private::Sealed for Vec<Expression> {}

impl Evaluate for Vec<Expression> {
    type Output = Self;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        self.into_iter().map(|expr| expr.evaluate(ctx)).collect()
    }
}

impl private::Sealed for Object<ObjectKey, Expression> {}

impl Evaluate for Object<ObjectKey, Expression> {
    type Output = Self;

    fn evaluate(self, ctx: &mut Context) -> EvalResult<Self::Output> {
        self.into_iter()
            .map(|(key, expr)| Ok((key.evaluate(ctx)?, expr.evaluate(ctx)?)))
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
            ElementAccessOperator::LegacyIndex(index) => eval_array_value(expr, index as usize),
            ElementAccessOperator::Index(index) => eval_index_expr(expr, index.evaluate(ctx)?),
            ElementAccessOperator::GetAttr(name) => eval_object_value(expr, name.into_inner()),
            ElementAccessOperator::AttrSplat => eval_attr_splat(expr),
            ElementAccessOperator::FullSplat => eval_full_splat(expr),
        }
    }
}

fn eval_index_expr(expr: Expression, index_expr: Expression) -> EvalResult<Expression> {
    match index_expr {
        Expression::String(name) => eval_object_value(expr, name),
        Expression::Number(num) => match num.as_u64() {
            Some(index) => eval_array_value(expr, index as usize),
            None => Err(EvalError::new(EvalErrorKind::UnexpectedExpression(
                Expression::Number(num),
                "an unsigned integer",
            ))),
        },
        other => Err(EvalError::new(EvalErrorKind::UnexpectedExpression(
            other,
            "a string or unsigned integer",
        ))),
    }
}

fn eval_array_value(expr: Expression, index: usize) -> EvalResult<Expression> {
    match expr {
        Expression::Array(mut array) => get_array_index(&mut array, index),
        other => Err(EvalError::new(EvalErrorKind::UnexpectedExpression(
            other, "an array",
        ))),
    }
}

fn eval_object_value(expr: Expression, name: String) -> EvalResult<Expression> {
    match expr {
        Expression::Object(mut object) => get_object_value(&mut object, ObjectKey::from(name)),
        other => Err(EvalError::new(EvalErrorKind::UnexpectedExpression(
            other,
            "an object",
        ))),
    }
}

fn eval_attr_splat(expr: Expression) -> EvalResult<Expression> {
    unimplemented!("evaluating attribute splat expression {expr} not implemented yet")
}

fn eval_full_splat(expr: Expression) -> EvalResult<Expression> {
    unimplemented!("evaluating full splat expression {expr} not implemented yet")
}

fn get_array_index(array: &mut Vec<Expression>, index: usize) -> EvalResult<Expression> {
    if index >= array.len() {
        return Err(EvalError::new(EvalErrorKind::IndexOutOfBounds(index)));
    }

    Ok(array.swap_remove(index))
}

fn get_object_value(
    object: &mut Object<ObjectKey, Expression>,
    key: ObjectKey,
) -> EvalResult<Expression> {
    match object.swap_remove(&key) {
        Some(value) => Ok(value),
        None => Err(EvalError::new(EvalErrorKind::NoSuchKey(key.to_string()))),
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
            other => Err(EvalError::new(EvalErrorKind::UnexpectedExpression(
                other,
                "a boolean",
            ))),
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
            (operator, expr) => Err(EvalError::from(format!(
                "operator `{}` cannot be applied to expression `{}`",
                operator.as_str(),
                expr
            ))),
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
                return Err(EvalError::new(EvalErrorKind::InvalidBinaryOp(
                    lhs, operator, rhs,
                )))
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
