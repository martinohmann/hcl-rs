use super::*;
use indexmap::map::Entry;

impl private::Sealed for Body {}

impl Evaluate for Body {
    type Output = Self;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        self.iter()
            .map(|structure| structure.evaluate(ctx))
            .collect::<Result<Body>>()
    }
}

impl private::Sealed for Structure {}

impl Evaluate for Structure {
    type Output = Self;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        match self {
            Structure::Attribute(attr) => attr.evaluate(ctx).map(Structure::Attribute),
            Structure::Block(block) => block.evaluate(ctx).map(Structure::Block),
        }
    }
}

impl private::Sealed for Attribute {}

impl Evaluate for Attribute {
    type Output = Self;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        Ok(Attribute {
            key: self.key.clone(),
            expr: self.expr.evaluate(ctx).map(Into::into)?,
        })
    }
}

impl private::Sealed for Block {}

impl Evaluate for Block {
    type Output = Self;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        Ok(Block {
            identifier: self.identifier.clone(),
            labels: self.labels.clone(),
            body: self.body.evaluate(ctx)?,
        })
    }
}

impl private::Sealed for Expression {}

impl Evaluate for Expression {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        match self {
            Expression::Array(array) => array.evaluate(ctx).map(Value::Array),
            Expression::Object(object) => object.evaluate(ctx).map(Value::Object),
            Expression::TemplateExpr(expr) => expr.evaluate(ctx).map(Value::String),
            Expression::Variable(ident) => ctx.get_var(ident).cloned(),
            Expression::Traversal(traversal) => traversal.evaluate(ctx),
            Expression::FuncCall(func_call) => func_call.evaluate(ctx),
            Expression::Parenthesis(expr) => expr.evaluate(ctx),
            Expression::Conditional(cond) => cond.evaluate(ctx),
            Expression::Operation(op) => op.evaluate(ctx),
            Expression::ForExpr(expr) => expr.evaluate(ctx),
            Expression::Raw(_) => Err(Error::new(ErrorKind::RawExpression)),
            other => Ok(Value::from(other.clone())),
        }
    }
}

impl private::Sealed for Vec<Expression> {}

impl Evaluate for Vec<Expression> {
    type Output = Vec<Value>;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        self.iter().map(|expr| expr.evaluate(ctx)).collect()
    }
}

impl private::Sealed for Object<ObjectKey, Expression> {}

impl Evaluate for Object<ObjectKey, Expression> {
    type Output = Map<String, Value>;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        self.iter()
            .map(|(key, expr)| Ok((key.evaluate(ctx)?, expr.evaluate(ctx)?)))
            .collect()
    }
}

impl private::Sealed for ObjectKey {}

impl Evaluate for ObjectKey {
    type Output = String;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        match self {
            ObjectKey::Expression(expr) => expr::evaluate_string(expr, ctx),
            ident => Ok(ident.to_string()),
        }
    }
}

impl private::Sealed for TemplateExpr {}

impl Evaluate for TemplateExpr {
    type Output = String;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        let template = Template::from_expr(self)?;
        template.evaluate(ctx)
    }
}

impl private::Sealed for Template {}

impl Evaluate for Template {
    type Output = String;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        let mut result = String::new();
        template::evaluate_template(&mut result, self, ctx, StripMode::None)?;
        Ok(result)
    }
}

impl private::Sealed for Traversal {}

impl Evaluate for Traversal {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        let value = self.expr.evaluate(ctx)?;
        expr::evaluate_traversal(value, self.operators.iter().cloned().collect(), ctx)
    }
}

impl private::Sealed for FuncCall {}

impl Evaluate for FuncCall {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        let func = ctx.get_func(&self.name)?;

        let len = self.args.len();
        let mut args = Vec::with_capacity(len);

        for (index, arg) in self.args.iter().enumerate() {
            if self.expand_final && index == len - 1 {
                let array = expr::evaluate_array(arg, ctx)?;
                args.extend(array);
            } else {
                args.push(arg.evaluate(ctx)?);
            }
        }

        func.call(args)
    }
}

impl private::Sealed for Conditional {}

impl Evaluate for Conditional {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        if expr::evaluate_bool(&self.cond_expr, ctx)? {
            self.true_expr.evaluate(ctx)
        } else {
            self.false_expr.evaluate(ctx)
        }
    }
}

impl private::Sealed for Operation {}

impl Evaluate for Operation {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        match self {
            Operation::Unary(unary) => unary.evaluate(ctx),
            Operation::Binary(binary) => binary.evaluate(ctx),
        }
    }
}

impl private::Sealed for UnaryOp {}

impl Evaluate for UnaryOp {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        let value = self.expr.evaluate(ctx)?;

        match (self.operator, value) {
            (UnaryOperator::Not, Value::Bool(v)) => Ok(Value::Bool(!v)),
            (UnaryOperator::Neg, Value::Number(n)) => Ok(Value::Number(-n)),
            (operator, value) => Err(Error::new(ErrorKind::InvalidUnaryOp(operator, value))),
        }
    }
}

impl private::Sealed for BinaryOp {}

impl Evaluate for BinaryOp {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        use {BinaryOperator::*, Value::*};

        let op = self.clone().normalize();
        let lhs = op.lhs_expr.evaluate(ctx)?;
        let rhs = op.rhs_expr.evaluate(ctx)?;

        let value = match (lhs, op.operator, rhs) {
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
                return Err(Error::new(ErrorKind::InvalidBinaryOp(lhs, operator, rhs)))
            }
        };

        Ok(value)
    }
}

impl private::Sealed for ForExpr {}

impl Evaluate for ForExpr {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> Result<Self::Output> {
        let collection = Collection::from_for_expr(self, ctx)?;

        match &self.key_expr {
            Some(key_expr) => {
                // Result will be an object.
                let mut result = Map::with_capacity(collection.len());

                for ctx in collection.into_iter() {
                    let ctx = &ctx?;
                    let key = expr::evaluate_string(key_expr, ctx)?;
                    let value = self.value_expr.evaluate(ctx)?;

                    if self.grouping {
                        match result
                            .entry(key)
                            .or_insert_with(|| Value::Array(Vec::new()))
                        {
                            Value::Array(array) => array.push(value),
                            _ => unreachable!(),
                        }
                    } else {
                        match result.entry(key) {
                            Entry::Occupied(entry) => {
                                return Err(Error::new(ErrorKind::KeyAlreadyExists(
                                    entry.key().clone(),
                                )))
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(value);
                            }
                        }
                    }
                }

                Ok(Value::Object(result))
            }
            None => {
                // Result will be an array.
                let mut result = Vec::with_capacity(collection.len());

                for ctx in collection.into_iter() {
                    result.push(self.value_expr.evaluate(&ctx?)?);
                }

                Ok(Value::Array(result))
            }
        }
    }
}
