use super::*;
use crate::expr::{
    BinaryOp, Conditional, Expression, ForExpr, FuncCall, FuncCallBuilder, Heredoc,
    HeredocStripMode, Object, ObjectKey, Operation, TemplateExpr, Traversal, TraversalOperator,
    UnaryOp, UnaryOperator, Variable,
};

pub fn expression(pair: Pair<Rule>) -> Result<Expression> {
    let pairs = pair.into_inner();
    let (expr, pairs) = unary_op(pairs)?;
    let (expr, pairs) = binary_op(expr, pairs)?;
    conditional(expr, pairs)
}

fn unary_op(mut pairs: Pairs<Rule>) -> Result<(Expression, Pairs<Rule>)> {
    let pair = pairs.next().unwrap();

    let expr = match pair.as_rule() {
        Rule::UnaryOperator => {
            let operator = from_str(pair);
            let expr = expr_term(pairs.next().unwrap())?;

            match (operator, expr) {
                (UnaryOperator::Neg, Expression::Number(num)) => Expression::Number(-num),
                (operator, expr) => Expression::from(Operation::Unary(UnaryOp { operator, expr })),
            }
        }
        _ => expr_term(pair)?,
    };

    Ok((expr, pairs))
}

fn binary_op(expr: Expression, mut pairs: Pairs<Rule>) -> Result<(Expression, Pairs<Rule>)> {
    let expr = match pairs.peek() {
        Some(pair) => match pair.as_rule() {
            Rule::BinaryOperator => Expression::from(Operation::Binary(BinaryOp {
                lhs_expr: expr,
                operator: from_str(pairs.next().unwrap()),
                rhs_expr: expression(pairs.next().unwrap())?,
            })),
            _ => expr,
        },
        None => expr,
    };

    Ok((expr, pairs))
}

fn conditional(expr: Expression, mut pairs: Pairs<Rule>) -> Result<Expression> {
    let expr = match pairs.next() {
        Some(pair) => Expression::from(Conditional {
            cond_expr: expr,
            true_expr: expression(pair)?,
            false_expr: expression(pairs.next().unwrap())?,
        }),
        None => expr,
    };

    Ok(expr)
}

fn expressions(pair: Pair<Rule>) -> Result<Vec<Expression>> {
    pair.into_inner().map(expression).collect()
}

fn expr_term(pair: Pair<Rule>) -> Result<Expression> {
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();

    let expr = match pair.as_rule() {
        Rule::BooleanLit => Expression::Bool(from_str(pair)),
        Rule::Float => {
            Number::from_f64(from_str::<f64>(pair)).map_or(Expression::Null, Expression::Number)
        }
        Rule::Int => Expression::from(from_str::<i64>(pair)),
        Rule::NullLit => Expression::Null,
        Rule::StringLit => unescape_string(inner(pair)).map(Expression::String)?,
        Rule::TemplateExpr => Expression::TemplateExpr(Box::new(template_expr(inner(pair)))),
        Rule::Tuple => expressions(pair).map(Expression::Array)?,
        Rule::Object => object(pair).map(Expression::Object)?,
        Rule::Variable => Expression::Variable(Variable::from(ident(pair))),
        Rule::FunctionCall => Expression::FuncCall(Box::new(func_call(pair)?)),
        Rule::Parenthesis => Expression::Parenthesis(Box::new(expression(inner(pair))?)),
        Rule::ForExpr => Expression::from(for_expr(inner(pair))?),
        rule => unexpected_rule(rule),
    };

    traversal(expr, pairs)
}

fn traversal(expr: Expression, pairs: Pairs<Rule>) -> Result<Expression> {
    let operators = pairs
        .map(traversal_operator)
        .collect::<Result<Vec<TraversalOperator>>>()?;

    if operators.is_empty() {
        Ok(expr)
    } else {
        Ok(Expression::from(Traversal { expr, operators }))
    }
}

fn for_expr(pair: Pair<Rule>) -> Result<ForExpr> {
    match pair.as_rule() {
        Rule::ForTupleExpr => for_list_expr(pair),
        Rule::ForObjectExpr => for_object_expr(pair),
        rule => unexpected_rule(rule),
    }
}

fn for_list_expr(pair: Pair<Rule>) -> Result<ForExpr> {
    let mut pairs = pair.into_inner();
    let intro = for_intro(pairs.next().unwrap())?;
    let value_expr = expression(pairs.next().unwrap())?;
    let cond_expr = match pairs.next() {
        Some(pair) => Some(expression(inner(pair))?),
        None => None,
    };

    Ok(ForExpr {
        key_var: intro.key_var,
        value_var: intro.value_var,
        collection_expr: intro.collection_expr,
        key_expr: None,
        value_expr,
        grouping: false,
        cond_expr,
    })
}

fn for_object_expr(pair: Pair<Rule>) -> Result<ForExpr> {
    let mut pairs = pair.into_inner();
    let intro = for_intro(pairs.next().unwrap())?;
    let key_expr = expression(pairs.next().unwrap())?;
    let value_expr = expression(pairs.next().unwrap())?;

    let (grouping, cond_expr) = match (pairs.next(), pairs.next()) {
        (Some(_), Some(pair)) => (true, Some(expression(inner(pair))?)),
        (Some(pair), None) => match pair.as_rule() {
            Rule::ValueGrouping => (true, None),
            Rule::ForCond => (false, Some(expression(inner(pair))?)),
            rule => unexpected_rule(rule),
        },
        (_, _) => (false, None),
    };

    Ok(ForExpr {
        key_var: intro.key_var,
        value_var: intro.value_var,
        collection_expr: intro.collection_expr,
        key_expr: Some(key_expr),
        value_expr,
        grouping,
        cond_expr,
    })
}

struct ForIntro {
    key_var: Option<Identifier>,
    value_var: Identifier,
    collection_expr: Expression,
}

fn for_intro(pair: Pair<Rule>) -> Result<ForIntro> {
    let mut pairs = pair.into_inner();
    let mut value_var = Some(ident(pairs.next().unwrap()));
    let mut expr = pairs.next().unwrap();

    // If there are two identifiers, the first one is the key and the second one the value.
    let key_var = match expr.as_rule() {
        Rule::Identifier => {
            let key = value_var.replace(ident(expr));
            expr = pairs.next().unwrap();
            key
        }
        _ => None,
    };

    Ok(ForIntro {
        key_var,
        value_var: value_var.take().unwrap(),
        collection_expr: expression(expr)?,
    })
}

fn func_call(pair: Pair<Rule>) -> Result<FuncCall> {
    let mut pairs = pair.into_inner();
    let builder = FuncCall::builder(ident(pairs.next().unwrap()));
    let mut args = pairs.next().unwrap().into_inner();

    args.try_fold(builder, |builder, pair| match pair.as_rule() {
        Rule::ExpandFinal => Ok(builder.expand_final(true)),
        _ => Ok(builder.arg(expression(pair)?)),
    })
    .map(FuncCallBuilder::build)
}

fn traversal_operator(pair: Pair<Rule>) -> Result<TraversalOperator> {
    let operator = match pair.as_rule() {
        Rule::AttrSplat => TraversalOperator::AttrSplat,
        Rule::FullSplat => TraversalOperator::FullSplat,
        Rule::GetAttr => TraversalOperator::GetAttr(ident(inner(pair))),
        Rule::Index => {
            let pair = inner(pair);

            match pair.as_rule() {
                Rule::LegacyIndex => TraversalOperator::LegacyIndex(from_str::<u64>(inner(pair))),
                _ => TraversalOperator::Index(expression(pair)?),
            }
        }
        rule => unexpected_rule(rule),
    };

    Ok(operator)
}

fn template_expr(pair: Pair<Rule>) -> TemplateExpr {
    match pair.as_rule() {
        Rule::QuotedStringTemplate => TemplateExpr::QuotedString(string(inner(pair))),
        Rule::Heredoc => TemplateExpr::Heredoc(heredoc(pair)),
        rule => unexpected_rule(rule),
    }
}

fn object(pair: Pair<Rule>) -> Result<Object<ObjectKey, Expression>> {
    ObjectIter::new(pair)
        .map(|(k, v)| Ok((object_key(k)?, expression(v)?)))
        .collect()
}

fn object_key(pair: Pair<Rule>) -> Result<ObjectKey> {
    match pair.as_rule() {
        Rule::Identifier => Ok(ObjectKey::Identifier(ident(pair))),
        _ => expression(pair).map(ObjectKey::Expression),
    }
}

fn heredoc(pair: Pair<Rule>) -> Heredoc {
    let mut pairs = pair.into_inner();
    let intro = pairs.next().unwrap();

    let strip = match intro.as_rule() {
        Rule::HeredocIntroNormal => HeredocStripMode::None,
        Rule::HeredocIntroIndent => HeredocStripMode::Indent,
        rule => unexpected_rule(rule),
    };

    let delimiter = ident(pairs.next().unwrap());
    let mut template = string(pairs.next().unwrap());

    // Append the trailing newline here. This is easier than doing this in the grammar.
    template.push('\n');

    Heredoc {
        delimiter,
        template,
        strip,
    }
}

struct ObjectIter<'a> {
    inner: Pairs<'a, Rule>,
}

impl<'a> ObjectIter<'a> {
    fn new(pair: Pair<'a, Rule>) -> Self {
        ObjectIter {
            inner: pair.into_inner(),
        }
    }
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = (Pair<'a, Rule>, Pair<'a, Rule>);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.inner.next(), self.inner.next()) {
            (Some(k), Some(v)) => Some((k, v)),
            (Some(k), None) => panic!("missing value for key: {}", k),
            (_, _) => None,
        }
    }
}
