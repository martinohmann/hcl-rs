use super::error::EvalResultExt;
use super::*;
use indexmap::map::Entry;
use std::hash::Hash;

impl private::Sealed for Body {}

impl Evaluate for Body {
    type Output = Self;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        self.iter()
            .map(|structure| structure.evaluate(ctx))
            .collect()
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        #[allow(clippy::manual_try_fold)]
        self.iter_mut().fold(Ok(()), |res, structure| {
            res.add_errors(structure.evaluate_in_place(ctx))
        })
    }
}

impl private::Sealed for Structure {}

impl Evaluate for Structure {
    type Output = Self;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        match self {
            Structure::Attribute(attr) => attr.evaluate(ctx).map(Structure::Attribute),
            Structure::Block(block) => block.evaluate(ctx).map(Structure::Block),
        }
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        match self {
            Structure::Attribute(attr) => attr.evaluate_in_place(ctx),
            Structure::Block(block) => block.evaluate_in_place(ctx),
        }
    }
}

impl private::Sealed for Attribute {}

impl Evaluate for Attribute {
    type Output = Self;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        Ok(Attribute {
            key: self.key.clone(),
            expr: self.expr.evaluate(ctx).map(Into::into)?,
        })
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        self.expr.evaluate_in_place(ctx)
    }
}

impl private::Sealed for Block {}

impl Evaluate for Block {
    type Output = Self;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        Ok(Block {
            identifier: self.identifier.clone(),
            labels: self.labels.clone(),
            body: self.body.evaluate(ctx)?,
        })
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        self.body.evaluate_in_place(ctx)
    }
}

impl private::Sealed for Expression {}

impl Evaluate for Expression {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        let ctx = &ctx.child_with_expr(self);
        match self {
            Expression::Array(array) => array.evaluate(ctx).map(Value::Array),
            Expression::Object(object) => object.evaluate(ctx).map(Value::Object),
            Expression::TemplateExpr(expr) => expr.evaluate(ctx),
            Expression::Variable(ident) => ctx.lookup_var(ident).cloned(),
            Expression::Traversal(traversal) => traversal.evaluate(ctx),
            Expression::FuncCall(func_call) => func_call.evaluate(ctx),
            Expression::Parenthesis(expr) => expr.evaluate(ctx),
            Expression::Conditional(cond) => cond.evaluate(ctx),
            Expression::Operation(op) => op.evaluate(ctx),
            Expression::ForExpr(expr) => expr.evaluate(ctx),
            other => Ok(Value::from(other.clone())),
        }
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        // We can ignore expressions that cannot be further evaluated. This avoids unnecessary
        // clone operations.
        if !matches!(
            self,
            Expression::Null | Expression::Bool(_) | Expression::Number(_) | Expression::String(_)
        ) {
            evaluate_nested_exprs(self, ctx)?;
            let value = self.evaluate(ctx)?;
            *self = value.into();
        }

        Ok(())
    }
}

fn evaluate_nested_exprs(expr: &mut Expression, ctx: &Context) -> EvalResult<(), Errors> {
    let expr_clone = expr.clone();
    let ctx = &ctx.child_with_expr(&expr_clone);

    match expr {
        Expression::Array(array) => array.evaluate_in_place(ctx),
        Expression::Object(object) => object.evaluate_in_place(ctx),
        Expression::Traversal(traversal) => traversal.evaluate_in_place(ctx),
        Expression::FuncCall(func_call) => func_call.evaluate_in_place(ctx),
        Expression::Parenthesis(expr) => expr.evaluate_in_place(ctx),
        Expression::Conditional(cond) => cond.evaluate_in_place(ctx),
        Expression::Operation(op) => op.evaluate_in_place(ctx),
        Expression::ForExpr(expr) => expr.evaluate_in_place(ctx),
        _ => Ok(()),
    }
}

impl<T> private::Sealed for Vec<T> where T: Evaluate {}

impl<T> Evaluate for Vec<T>
where
    T: Evaluate,
{
    type Output = Vec<T::Output>;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        self.iter().map(|expr| expr.evaluate(ctx)).collect()
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        #[allow(clippy::manual_try_fold)]
        self.iter_mut().fold(Ok(()), |res, element| {
            res.add_errors(element.evaluate_in_place(ctx))
        })
    }
}

impl<K, V> private::Sealed for Object<K, V>
where
    K: Evaluate,
    V: Evaluate,
{
}

impl<K, V> Evaluate for Object<K, V>
where
    K: Evaluate + Eq,
    K::Output: Hash + Eq,
    V: Evaluate,
{
    type Output = Map<K::Output, V::Output>;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        self.iter()
            .map(|(key, expr)| Ok((key.evaluate(ctx)?, expr.evaluate(ctx)?)))
            .collect()
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        let mut new_object = Object::with_capacity(self.len());

        #[allow(clippy::manual_try_fold)]
        let res = self
            .drain(..)
            .fold(Ok(()), |mut res, (mut key, mut value)| {
                res = res
                    .add_errors(key.evaluate_in_place(ctx))
                    .add_errors(value.evaluate_in_place(ctx));
                new_object.insert(key, value);
                res
            });

        *self = new_object;

        res
    }
}

impl private::Sealed for ObjectKey {}

impl Evaluate for ObjectKey {
    type Output = String;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        match self {
            ObjectKey::Expression(expr) => expr::evaluate_object_key(expr, ctx),
            ident => Ok(ident.to_string()),
        }
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        if let ObjectKey::Expression(expr) = self {
            expr.evaluate_in_place(ctx)?;
        }

        Ok(())
    }
}

impl private::Sealed for TemplateExpr {}

impl Evaluate for TemplateExpr {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        let template = Template::from_expr(self)?;
        let elements = template.elements();

        // If the template consists only of a single interpolation, with no surrounding literals,
        // directives or other interpolations, perform interpolation unwrapping as described in the
        // spec:
        //
        // https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md#template-interpolation-unwrapping
        match elements.first() {
            Some(Element::Interpolation(interp)) if elements.len() == 1 => {
                interp.expr.evaluate(ctx)
            }
            _ => template.evaluate(ctx).map(Value::String),
        }
    }
}

impl private::Sealed for Template {}

impl Evaluate for Template {
    type Output = String;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        let mut result = String::new();
        template::evaluate_template(&mut result, self, ctx, Strip::None, Strip::None)?;
        Ok(result)
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        #[allow(clippy::manual_try_fold)]
        self.elements_mut()
            .iter_mut()
            .fold(Ok(()), |mut res, element| match element {
                Element::Literal(_) => res,
                Element::Interpolation(interp) => {
                    res.add_errors(interp.expr.evaluate_in_place(ctx))
                }
                Element::Directive(dir) => match dir {
                    Directive::If(dir) => {
                        res = res
                            .add_errors(dir.cond_expr.evaluate_in_place(ctx))
                            .add_errors(dir.true_template.evaluate_in_place(ctx));

                        match &mut dir.false_template {
                            Some(false_template) => {
                                res.add_errors(false_template.evaluate_in_place(ctx))
                            }
                            None => res,
                        }
                    }
                    Directive::For(dir) => res
                        .add_errors(dir.collection_expr.evaluate_in_place(ctx))
                        .add_errors(dir.template.evaluate_in_place(ctx)),
                },
            })
    }
}

impl private::Sealed for Traversal {}

impl Evaluate for Traversal {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        let value = self.expr.evaluate(ctx)?;
        let deque = self.operators.iter().collect();
        expr::evaluate_traversal(value, deque, ctx)
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        #[allow(clippy::manual_try_fold)]
        self.operators
            .iter_mut()
            .fold(Ok(()), |res, operator| match operator {
                TraversalOperator::Index(expr) => res.add_errors(expr.evaluate_in_place(ctx)),
                _ => res,
            })
    }
}

impl private::Sealed for FuncCall {}

impl Evaluate for FuncCall {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        let name = &self.name;
        let func = ctx.lookup_func(name)?;
        let len = self.args.len();
        let mut args = Vec::with_capacity(len);

        for (index, arg) in self.args.iter().enumerate() {
            if self.expand_final && index == len - 1 {
                args.extend(expr::evaluate_array(arg, ctx)?);
            } else {
                args.push(arg.evaluate(ctx)?);
            }
        }

        func.call(args)
            .map_err(|err| ctx.error(ErrorKind::FuncCall(name.clone(), err)))
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        self.args.evaluate_in_place(ctx)
    }
}

impl private::Sealed for Conditional {}

impl Evaluate for Conditional {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        if expr::evaluate_bool(&self.cond_expr, ctx)? {
            self.true_expr.evaluate(ctx)
        } else {
            self.false_expr.evaluate(ctx)
        }
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        let cond = expr::evaluate_bool(&self.cond_expr, ctx)?;

        self.cond_expr = Expression::from(cond);

        if cond {
            self.true_expr.evaluate_in_place(ctx)
        } else {
            self.false_expr.evaluate_in_place(ctx)
        }
    }
}

impl private::Sealed for Operation {}

impl Evaluate for Operation {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        match self {
            Operation::Unary(unary) => unary.evaluate(ctx),
            Operation::Binary(binary) => binary.evaluate(ctx),
        }
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        match self {
            Operation::Unary(unary) => unary.evaluate_in_place(ctx),
            Operation::Binary(binary) => binary.evaluate_in_place(ctx),
        }
    }
}

impl private::Sealed for UnaryOp {}

impl Evaluate for UnaryOp {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        use {UnaryOperator::*, Value::*};

        let value = self.expr.evaluate(ctx)?;

        let value = match (self.operator, value) {
            (Not, Bool(v)) => Bool(!v),
            (Neg, Number(n)) => Number(-n),
            (operator, value) => return Err(ctx.error(ErrorKind::UnaryOp(operator, value))),
        };

        Ok(value)
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        self.expr.evaluate_in_place(ctx)
    }
}

impl private::Sealed for BinaryOp {}

impl Evaluate for BinaryOp {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
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
            (lhs, operator, rhs) => return Err(ctx.error(ErrorKind::BinaryOp(lhs, operator, rhs))),
        };

        Ok(value)
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        self.lhs_expr
            .evaluate_in_place(ctx)
            .add_errors(self.rhs_expr.evaluate_in_place(ctx))
    }
}

impl private::Sealed for ForExpr {}

impl Evaluate for ForExpr {
    type Output = Value;

    fn evaluate(&self, ctx: &Context) -> EvalResult<Self::Output> {
        let collection = expr::Collection::from_for_expr(self, ctx)?;

        match &self.key_expr {
            Some(key_expr) => {
                // Result will be an object.
                let mut result = Map::with_capacity(collection.len());

                for ctx in collection {
                    let ctx = &ctx?;
                    let key = expr::evaluate_object_key(key_expr, ctx)?;
                    let value = self.value_expr.evaluate(ctx)?;

                    if self.grouping {
                        result
                            .entry(key)
                            .or_insert_with(|| Value::Array(Vec::new()))
                            .as_array_mut()
                            .unwrap()
                            .push(value);
                    } else {
                        match result.entry(key) {
                            Entry::Occupied(entry) => {
                                return Err(ctx.error(ErrorKind::KeyExists(entry.key().clone())))
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
                let result = collection
                    .into_iter()
                    .map(|ctx| self.value_expr.evaluate(&ctx?))
                    .collect::<EvalResult<_>>()?;

                Ok(Value::Array(result))
            }
        }
    }

    fn evaluate_in_place(&mut self, ctx: &Context) -> EvalResult<(), Errors> {
        self.collection_expr.evaluate_in_place(ctx)
    }
}
