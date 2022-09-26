use super::{for_expr::Collection, *};
use crate::{structure::*, template::Template};
use std::collections::VecDeque;
use vecmap::map::Entry;

impl private::Sealed for Body {}

impl Evaluate for Body {
    type Output = Self;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        self.into_iter()
            .map(|structure| structure.evaluate(ctx))
            .collect::<EvalResult<Body>>()
    }
}

impl private::Sealed for Structure {}

impl Evaluate for Structure {
    type Output = Self;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        match self {
            Structure::Attribute(attr) => attr.evaluate(ctx).map(Structure::Attribute),
            Structure::Block(block) => block.evaluate(ctx).map(Structure::Block),
        }
    }
}

impl private::Sealed for Attribute {}

impl Evaluate for Attribute {
    type Output = Self;

    fn evaluate(mut self, ctx: &Context) -> EvalResult<Self::Output> {
        self.expr = self.expr.evaluate(ctx)?;
        Ok(self)
    }
}

impl private::Sealed for Block {}

impl Evaluate for Block {
    type Output = Self;

    fn evaluate(mut self, ctx: &Context) -> EvalResult<Self::Output> {
        self.body = self.body.evaluate(ctx)?;
        Ok(self)
    }
}

impl private::Sealed for Expression {}

impl Evaluate for Expression {
    type Output = Self;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        match self {
            Expression::Array(array) => array.evaluate(ctx).map(Expression::Array),
            Expression::Object(object) => object.evaluate(ctx).map(Expression::Object),
            Expression::TemplateExpr(expr) => expr.evaluate(ctx).map(Expression::String),
            Expression::VariableExpr(ident) => {
                ctx.get_variable(ident.as_str()).cloned().map(Into::into)
            }
            Expression::Traversal(traversal) => traversal.evaluate(ctx),
            Expression::FuncCall(func_call) => func_call.evaluate(ctx),
            Expression::SubExpr(expr) => expr.evaluate(ctx),
            Expression::Conditional(cond) => cond.evaluate(ctx),
            Expression::Operation(op) => op.evaluate(ctx),
            Expression::ForExpr(expr) => expr.evaluate(ctx),
            Expression::Raw(_) => Err(ctx.error(EvalErrorKind::RawExpression)),
            other => Ok(other),
        }
    }
}

impl private::Sealed for Vec<Expression> {}

impl Evaluate for Vec<Expression> {
    type Output = Self;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        self.into_iter().map(|expr| expr.evaluate(ctx)).collect()
    }
}

impl private::Sealed for Object<ObjectKey, Expression> {}

impl Evaluate for Object<ObjectKey, Expression> {
    type Output = Self;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        self.into_iter()
            .map(|(key, expr)| Ok((key.evaluate(ctx)?, expr.evaluate(ctx)?)))
            .collect()
    }
}

impl private::Sealed for ObjectKey {}

impl Evaluate for ObjectKey {
    type Output = Self;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        match self {
            ObjectKey::Expression(expr) => expr.evaluate(ctx).map(ObjectKey::Expression),
            ident => Ok(ident),
        }
    }
}

impl private::Sealed for TemplateExpr {}

impl Evaluate for TemplateExpr {
    type Output = String;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        match Template::from_expr(&self) {
            Ok(template) => template.evaluate(ctx),
            Err(err) => Err(err.into()),
        }
    }
}

impl private::Sealed for Template {}

impl Evaluate for Template {
    type Output = String;

    fn evaluate(self, _ctx: &Context) -> EvalResult<Self::Output> {
        todo!()
    }
}

impl private::Sealed for Traversal {}

impl Evaluate for Traversal {
    type Output = Expression;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        expr::evaluate_traversal(self.expr, VecDeque::from(self.operators), ctx)
    }
}

impl private::Sealed for FuncCall {}

impl Evaluate for FuncCall {
    type Output = Expression;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        let func = ctx.get_func(self.name.as_str())?;

        let len = self.args.len();
        let mut args = Vec::with_capacity(len);

        for (index, arg) in self.args.into_iter().enumerate() {
            if self.expand_final && index == len - 1 {
                let array = expr::evaluate_array(arg, ctx)?;
                args.extend(array);
            } else {
                args.push(arg.evaluate(ctx)?);
            }
        }

        func(args)
    }
}

impl private::Sealed for Conditional {}

impl Evaluate for Conditional {
    type Output = Expression;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        if expr::evaluate_bool(self.cond_expr, ctx)? {
            self.true_expr.evaluate(ctx)
        } else {
            self.false_expr.evaluate(ctx)
        }
    }
}

impl private::Sealed for Operation {}

impl Evaluate for Operation {
    type Output = Expression;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        match self {
            Operation::Unary(unary) => unary.evaluate(ctx),
            Operation::Binary(binary) => binary.evaluate(ctx),
        }
    }
}

impl private::Sealed for UnaryOp {}

impl Evaluate for UnaryOp {
    type Output = Expression;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
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

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
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
            (Number(lhs), Plus, Number(rhs)) => Number(lhs + rhs),
            (Number(lhs), Minus, Number(rhs)) => Number(lhs - rhs),
            (Number(lhs), Mul, Number(rhs)) => Number(lhs * rhs),
            (Number(lhs), Div, Number(rhs)) => Number(lhs / rhs),
            (Number(lhs), Mod, Number(rhs)) => Number(lhs % rhs),
            (lhs, operator, rhs) => {
                return Err(ctx.error(EvalErrorKind::InvalidBinaryOp(lhs, operator, rhs)))
            }
        };

        Ok(expr)
    }
}

impl private::Sealed for ForExpr {}

impl Evaluate for ForExpr {
    type Output = Expression;

    fn evaluate(self, ctx: &Context) -> EvalResult<Self::Output> {
        let collection = Collection::new(&self, ctx)?;

        match &self.key_expr {
            Some(key_expr) => {
                // Result will be an object.
                let mut result = Object::with_capacity(collection.len());

                for ctx in collection.into_iter() {
                    let ctx = &ctx?;
                    let key = key_expr.clone().evaluate(ctx).map(ObjectKey::from)?;
                    let value = self.value_expr.clone().evaluate(ctx)?;

                    if self.grouping {
                        match result
                            .entry(key)
                            .or_insert_with(|| Expression::Array(Vec::new()))
                        {
                            Expression::Array(array) => array.push(value),
                            _ => unreachable!(),
                        }
                    } else {
                        match result.entry(key) {
                            Entry::Occupied(entry) => {
                                return Err(
                                    ctx.error(EvalErrorKind::KeyAlreadyExists(entry.into_key()))
                                )
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(value);
                            }
                        }
                    }
                }

                Ok(Expression::Object(result))
            }
            None => {
                // Result will be an array.
                let mut result = Vec::with_capacity(collection.len());

                for ctx in collection.into_iter() {
                    result.push(self.value_expr.clone().evaluate(&ctx?)?);
                }

                Ok(Expression::Array(result))
            }
        }
    }
}
