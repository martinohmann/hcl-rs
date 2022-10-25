mod template;
#[cfg(test)]
mod tests;

pub use self::template::parse as parse_template;
use crate::{structure::*, util::unescape, Number, Result};
use pest::{
    iterators::{Pair, Pairs},
    Parser as ParserTrait,
};
use pest_derive::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "parser/grammar/hcl.pest"]
struct HclParser;

/// Parse a `hcl::Body` from a `&str`.
///
/// If deserialization into a different type is preferred consider using [`hcl::from_str`][from_str].
///
/// [from_str]: ./de/fn.from_str.html
///
/// # Example
///
/// ```
/// use hcl::{Attribute, Block, Body};
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let input = r#"
///     some_attr = "foo"
///
///     some_block "some_block_label" {
///       attr = "value"
///     }
/// "#;
///
/// let expected = Body::builder()
///     .add_attribute(("some_attr", "foo"))
///     .add_block(
///         Block::builder("some_block")
///             .add_label("some_block_label")
///             .add_attribute(("attr", "value"))
///             .build()
///     )
///     .build();
///
/// let body = hcl::parse(input)?;
///
/// assert_eq!(body, expected);
/// #   Ok(())
/// # }
/// ```
pub fn parse(input: &str) -> Result<Body> {
    let pair = HclParser::parse(Rule::Hcl, input)?.next().unwrap();
    parse_body(pair)
}

fn parse_body(pair: Pair<Rule>) -> Result<Body> {
    pair.into_inner().map(parse_structure).collect()
}

fn parse_structure(pair: Pair<Rule>) -> Result<Structure> {
    match pair.as_rule() {
        Rule::Attribute => parse_attribute(pair).map(Structure::Attribute),
        Rule::Block => parse_block(pair).map(Structure::Block),
        rule => unexpected_rule(rule),
    }
}

fn parse_attribute(pair: Pair<Rule>) -> Result<Attribute> {
    let mut pairs = pair.into_inner();

    Ok(Attribute {
        key: parse_ident(pairs.next().unwrap()).into_inner(),
        expr: parse_expression(pairs.next().unwrap())?,
    })
}

fn parse_block(pair: Pair<Rule>) -> Result<Block> {
    let mut pairs = pair.into_inner();

    let identifier = parse_ident(pairs.next().unwrap()).into_inner();

    let (labels, block_body): (Vec<Pair<Rule>>, Vec<Pair<Rule>>) =
        pairs.partition(|pair| pair.as_rule() != Rule::BlockBody);

    Ok(Block {
        identifier,
        labels: labels
            .into_iter()
            .map(parse_block_label)
            .collect::<Result<_>>()?,
        body: parse_block_body(block_body.into_iter().next().unwrap())?,
    })
}

fn parse_block_label(pair: Pair<Rule>) -> Result<BlockLabel> {
    match pair.as_rule() {
        Rule::Identifier => Ok(BlockLabel::Identifier(parse_ident(pair))),
        Rule::StringLit => parse_string(inner(pair)).map(BlockLabel::String),
        rule => unexpected_rule(rule),
    }
}

fn parse_block_body(pair: Pair<Rule>) -> Result<Body> {
    match pair.as_rule() {
        Rule::BlockBody => parse_body(inner(pair)),
        rule => unexpected_rule(rule),
    }
}

fn parse_expression(pair: Pair<Rule>) -> Result<Expression> {
    let pairs = pair.into_inner();
    let (expr, pairs) = parse_unary_op(pairs)?;
    let (expr, pairs) = parse_binary_op(expr, pairs)?;
    parse_conditional(expr, pairs)
}

fn parse_unary_op(mut pairs: Pairs<Rule>) -> Result<(Expression, Pairs<Rule>)> {
    let pair = pairs.next().unwrap();

    let expr = match pair.as_rule() {
        Rule::UnaryOperator => {
            let operator = UnaryOperator::from_str(pair.as_str())?;
            let expr = parse_expr_term(pairs.next().unwrap())?;

            match (operator, expr) {
                (UnaryOperator::Neg, Expression::Number(num)) => Expression::Number(-num),
                (operator, expr) => Expression::from(Operation::Unary(UnaryOp { operator, expr })),
            }
        }
        _ => parse_expr_term(pair)?,
    };

    Ok((expr, pairs))
}

fn parse_binary_op(expr: Expression, mut pairs: Pairs<Rule>) -> Result<(Expression, Pairs<Rule>)> {
    let expr = match pairs.peek() {
        Some(pair) => match pair.as_rule() {
            Rule::BinaryOperator => Expression::from(Operation::Binary(BinaryOp {
                lhs_expr: expr,
                operator: pairs.next().unwrap().as_str().parse()?,
                rhs_expr: parse_expression(pairs.next().unwrap())?,
            })),
            _ => expr,
        },
        None => expr,
    };

    Ok((expr, pairs))
}

fn parse_conditional(expr: Expression, mut pairs: Pairs<Rule>) -> Result<Expression> {
    let expr = match pairs.next() {
        Some(pair) => Expression::from(Conditional {
            cond_expr: expr,
            true_expr: parse_expression(pair)?,
            false_expr: parse_expression(pairs.next().unwrap())?,
        }),
        None => expr,
    };

    Ok(expr)
}

fn parse_expressions(pair: Pair<Rule>) -> Result<Vec<Expression>> {
    pair.into_inner().map(parse_expression).collect()
}

fn parse_expr_term(pair: Pair<Rule>) -> Result<Expression> {
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();

    let expr = match pair.as_rule() {
        Rule::BooleanLit => Expression::Bool(parse_primitive(pair)),
        Rule::Float => Number::from_f64(parse_primitive::<f64>(pair))
            .map_or(Expression::Null, Expression::Number),
        Rule::Int => Expression::Number(parse_primitive::<i64>(pair).into()),
        Rule::NullLit => Expression::Null,
        Rule::StringLit => parse_string(inner(pair)).map(Expression::String)?,
        Rule::TemplateExpr => Expression::TemplateExpr(Box::new(parse_template_expr(inner(pair)))),
        Rule::Tuple => parse_expressions(pair).map(Expression::Array)?,
        Rule::Object => parse_object(pair).map(Expression::Object)?,
        Rule::Variable => Expression::Variable(parse_ident(pair)),
        Rule::FunctionCall => Expression::FuncCall(Box::new(parse_func_call(pair)?)),
        Rule::Parenthesis => Expression::Parenthesis(Box::new(parse_expression(inner(pair))?)),
        Rule::ForExpr => Expression::from(parse_for_expr(inner(pair))?),
        rule => unexpected_rule(rule),
    };

    parse_traversal(expr, pairs)
}

fn parse_traversal(expr: Expression, pairs: Pairs<Rule>) -> Result<Expression> {
    let operators = pairs
        .map(parse_traversal_operator)
        .collect::<Result<Vec<TraversalOperator>>>()?;

    if !operators.is_empty() {
        Ok(Expression::from(Traversal { expr, operators }))
    } else {
        Ok(expr)
    }
}

fn parse_for_expr(pair: Pair<Rule>) -> Result<ForExpr> {
    match pair.as_rule() {
        Rule::ForTupleExpr => parse_for_list_expr(pair),
        Rule::ForObjectExpr => parse_for_object_expr(pair),
        rule => unexpected_rule(rule),
    }
}

fn parse_for_list_expr(pair: Pair<Rule>) -> Result<ForExpr> {
    let mut pairs = pair.into_inner();
    let (key_var, value_var, collection_expr) = parse_for_intro(pairs.next().unwrap())?;
    let value_expr = parse_expression(pairs.next().unwrap())?;
    let cond_expr = match pairs.next() {
        Some(pair) => Some(parse_expression(inner(pair))?),
        None => None,
    };

    Ok(ForExpr {
        key_var,
        value_var,
        collection_expr,
        key_expr: None,
        value_expr,
        grouping: false,
        cond_expr,
    })
}

fn parse_for_object_expr(pair: Pair<Rule>) -> Result<ForExpr> {
    let mut pairs = pair.into_inner();
    let (key_var, value_var, collection_expr) = parse_for_intro(pairs.next().unwrap())?;
    let key_expr = parse_expression(pairs.next().unwrap())?;
    let value_expr = parse_expression(pairs.next().unwrap())?;
    let (grouping, cond_expr) = match (pairs.next(), pairs.next()) {
        (Some(_), Some(pair)) => (true, Some(parse_expression(inner(pair))?)),
        (Some(pair), None) => match pair.as_rule() {
            Rule::ValueGrouping => (true, None),
            Rule::ForCond => (false, Some(parse_expression(inner(pair))?)),
            rule => unexpected_rule(rule),
        },
        (_, _) => (false, None),
    };

    Ok(ForExpr {
        key_var,
        value_var,
        collection_expr,
        key_expr: Some(key_expr),
        value_expr,
        grouping,
        cond_expr,
    })
}

fn parse_for_intro(pair: Pair<Rule>) -> Result<(Option<Identifier>, Identifier, Expression)> {
    let mut pairs = pair.into_inner();
    let value = pairs.next().unwrap();
    let mut value_var = Some(parse_ident(value));
    let mut expr = pairs.next().unwrap();

    // If there are two identifiers, the first one is the key and the second one the value.
    let key_var = match expr.as_rule() {
        Rule::Identifier => {
            let key = value_var.replace(parse_ident(expr));
            expr = pairs.next().unwrap();
            key
        }
        _ => None,
    };

    Ok((key_var, value_var.take().unwrap(), parse_expression(expr)?))
}

fn parse_func_call(pair: Pair<Rule>) -> Result<FuncCall> {
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap();
    let mut args = pairs.next().unwrap().into_inner();
    let builder = FuncCall::builder(name.as_str());

    args.try_fold(builder, |builder, pair| match pair.as_rule() {
        Rule::ExpandFinal => Ok(builder.expand_final(true)),
        _ => Ok(builder.arg(parse_expression(pair)?)),
    })
    .map(FuncCallBuilder::build)
}

fn parse_traversal_operator(pair: Pair<Rule>) -> Result<TraversalOperator> {
    let operator = match pair.as_rule() {
        Rule::AttrSplat => TraversalOperator::AttrSplat,
        Rule::FullSplat => TraversalOperator::FullSplat,
        Rule::GetAttr => TraversalOperator::GetAttr(parse_ident(inner(pair))),
        Rule::Index => {
            let pair = inner(pair);

            match pair.as_rule() {
                Rule::LegacyIndex => {
                    TraversalOperator::LegacyIndex(parse_primitive::<u64>(inner(pair)))
                }
                _ => TraversalOperator::Index(parse_expression(pair)?),
            }
        }
        rule => unexpected_rule(rule),
    };

    Ok(operator)
}

fn parse_template_expr(pair: Pair<Rule>) -> TemplateExpr {
    match pair.as_rule() {
        Rule::QuotedStringTemplate => {
            let pair = inner(pair);
            TemplateExpr::QuotedString(pair.as_str().to_owned())
        }
        Rule::HeredocTemplate => TemplateExpr::Heredoc(parse_heredoc(pair)),
        rule => unexpected_rule(rule),
    }
}

fn parse_object(pair: Pair<Rule>) -> Result<Object<ObjectKey, Expression>> {
    ObjectIter::new(pair)
        .map(|(k, v)| Ok((parse_object_key(k)?, parse_expression(v)?)))
        .collect()
}

fn parse_object_key(pair: Pair<Rule>) -> Result<ObjectKey> {
    match pair.as_rule() {
        Rule::Identifier => Ok(ObjectKey::Identifier(parse_ident(pair))),
        _ => parse_expression(pair).map(ObjectKey::Expression),
    }
}

fn parse_primitive<F>(pair: Pair<Rule>) -> F
where
    F: FromStr,
    <F as FromStr>::Err: std::fmt::Debug,
{
    pair.as_str().parse::<F>().unwrap()
}

fn inner(pair: Pair<Rule>) -> Pair<Rule> {
    pair.into_inner().next().unwrap()
}

fn parse_string(pair: Pair<Rule>) -> Result<String> {
    unescape(pair.as_str()).map(|c| c.to_string())
}

fn parse_ident(pair: Pair<Rule>) -> Identifier {
    Identifier::new(pair.as_str())
}

fn parse_heredoc(pair: Pair<Rule>) -> Heredoc {
    let mut pairs = pair.into_inner();
    let intro = pairs.next().unwrap();
    let delimiter = pairs.next().unwrap();
    let template = pairs.next().unwrap();

    let strip = match intro.as_rule() {
        Rule::HeredocIntroNormal => HeredocStripMode::None,
        Rule::HeredocIntroIndent => HeredocStripMode::Indent,
        rule => unexpected_rule(rule),
    };

    Heredoc {
        strip,
        delimiter: parse_ident(delimiter),
        template: template.as_str().to_owned(),
    }
}

#[track_caller]
fn unexpected_rule(rule: Rule) -> ! {
    panic!("unexpected rule: {:?}", rule)
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
