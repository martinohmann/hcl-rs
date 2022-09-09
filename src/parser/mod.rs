mod template;
#[cfg(test)]
mod tests;

pub use self::template::parse as parse_template;
use crate::{structure::*, util::unescape, Result};
use pest::{
    iterators::{Pair, Pairs},
    Parser as ParserTrait,
};
use pest_derive::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "parser/grammar/hcl.pest"]
struct HclParser;

/// Parses a HCL `Body` from a `&str`.
///
/// If deserialization into a different type is preferred consider using [`hcl::from_str`][from_str].
///
/// [from_str]: ./de/fn.from_str.html
///
/// ## Example
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
        key: parse_ident(pairs.next().unwrap()),
        expr: parse_expression(pairs.next().unwrap())?,
    })
}

fn parse_block(pair: Pair<Rule>) -> Result<Block> {
    let mut pairs = pair.into_inner();

    let identifier = parse_ident(pairs.next().unwrap());

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
        Rule::Identifier => Ok(BlockLabel::identifier(parse_ident(pair))),
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
    match pair.as_rule() {
        Rule::ExprTerm => parse_expr_term(pair),
        Rule::Operation => match parse_operation(inner(pair))? {
            Operation::Binary(binary) => Ok(Expression::from(Operation::Binary(binary))),
            Operation::Unary(unary) => match (unary.operator, unary.expr) {
                // Negate operations on numbers are just converted to negative numbers for
                // convenience.
                (UnaryOperator::Neg, Expression::Number(num)) => Ok(Expression::Number(-num)),
                (operator, expr) => Ok(Expression::from(Operation::Unary(UnaryOp {
                    operator,
                    expr,
                }))),
            },
        },
        Rule::Conditional => Ok(Expression::from(parse_conditional(pair)?)),
        rule => unexpected_rule(rule),
    }
}

fn parse_conditional(pair: Pair<Rule>) -> Result<Conditional> {
    let mut pairs = pair.into_inner();

    Ok(Conditional {
        predicate: parse_expression(pairs.next().unwrap())?,
        true_expr: parse_expression(pairs.next().unwrap())?,
        false_expr: parse_expression(pairs.next().unwrap())?,
    })
}

fn parse_operation(pair: Pair<Rule>) -> Result<Operation> {
    match pair.as_rule() {
        Rule::UnaryOp => parse_unary_op(pair).map(Operation::Unary),
        Rule::BinaryOp => parse_binary_op(pair).map(Operation::Binary),
        rule => unexpected_rule(rule),
    }
}

fn parse_unary_op(pair: Pair<Rule>) -> Result<UnaryOp> {
    let mut pairs = pair.into_inner();

    Ok(UnaryOp {
        operator: pairs.next().unwrap().as_str().parse()?,
        expr: parse_expression(pairs.next().unwrap())?,
    })
}

fn parse_binary_op(pair: Pair<Rule>) -> Result<BinaryOp> {
    let mut pairs = pair.into_inner();

    Ok(BinaryOp {
        lhs_expr: parse_expr_term(pairs.next().unwrap())?,
        operator: pairs.next().unwrap().as_str().parse()?,
        rhs_expr: parse_expression(pairs.next().unwrap())?,
    })
}

fn parse_expressions(pair: Pair<Rule>) -> Result<Vec<Expression>> {
    pair.into_inner().map(parse_expression).collect()
}

fn parse_expr_term(pair: Pair<Rule>) -> Result<Expression> {
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();

    let expr = match pair.as_rule() {
        Rule::BooleanLit => Expression::Bool(parse_primitive(pair)),
        Rule::Float => Expression::Number(parse_primitive::<f64>(pair).into()),
        Rule::Int => Expression::Number(parse_primitive::<i64>(pair).into()),
        Rule::NullLit => Expression::Null,
        Rule::StringLit => parse_string(inner(pair)).map(Expression::String)?,
        Rule::TemplateExpr => {
            let expr = parse_template_expr(inner(pair));
            Expression::TemplateExpr(Box::new(expr))
        }
        Rule::Tuple => parse_expressions(pair).map(Expression::Array)?,
        Rule::Object => parse_object(pair).map(Expression::Object)?,
        Rule::VariableExpr => Expression::VariableExpr(parse_ident(pair).into()),
        Rule::FunctionCall => Expression::FuncCall(Box::new(parse_func_call(pair)?)),
        Rule::SubExpression => Expression::SubExpr(Box::new(parse_expression(inner(pair))?)),
        Rule::ForExpr => Expression::from(parse_for_expr(inner(pair))?),
        rule => unexpected_rule(rule),
    };

    pairs.try_fold(expr, |expr, pair| {
        Ok(expr.element(parse_element_access_operator(pair)?))
    })
}

fn parse_for_expr(pair: Pair<Rule>) -> Result<ForExpr> {
    match pair.as_rule() {
        Rule::ForTupleExpr => parse_for_list_expr(pair).map(ForExpr::List),
        Rule::ForObjectExpr => parse_for_object_expr(pair).map(ForExpr::Object),
        rule => unexpected_rule(rule),
    }
}

fn parse_for_list_expr(pair: Pair<Rule>) -> Result<ForListExpr> {
    let mut pairs = pair.into_inner();
    let intro = parse_for_intro(pairs.next().unwrap())?;
    let expr = parse_expression(pairs.next().unwrap())?;
    let cond = match pairs.next() {
        Some(pair) => Some(parse_expression(inner(pair))?),
        None => None,
    };

    Ok(ForListExpr { intro, expr, cond })
}

fn parse_for_object_expr(pair: Pair<Rule>) -> Result<ForObjectExpr> {
    let mut pairs = pair.into_inner();
    let intro = parse_for_intro(pairs.next().unwrap())?;
    let key_expr = parse_expression(pairs.next().unwrap())?;
    let value_expr = parse_expression(pairs.next().unwrap())?;
    let (value_grouping, cond) = match (pairs.next(), pairs.next()) {
        (Some(_), Some(pair)) => (true, Some(parse_expression(inner(pair))?)),
        (Some(pair), None) => match pair.as_rule() {
            Rule::ValueGrouping => (true, None),
            Rule::ForCond => (false, Some(parse_expression(inner(pair))?)),
            rule => unexpected_rule(rule),
        },
        (_, _) => (false, None),
    };

    Ok(ForObjectExpr {
        intro,
        key_expr,
        value_expr,
        value_grouping,
        cond,
    })
}

fn parse_for_intro(pair: Pair<Rule>) -> Result<ForIntro> {
    let mut pairs = pair.into_inner();
    let value = pairs.next().unwrap();
    let mut value = Some(Identifier::new(value.as_str()));
    let mut expr = pairs.next().unwrap();

    // If there are two identifiers, the first one is the key and the second one the value.
    let key = match expr.as_rule() {
        Rule::Identifier => {
            let key = value.replace(Identifier::new(expr.as_str()));
            expr = pairs.next().unwrap();
            key
        }
        _ => None,
    };

    Ok(ForIntro {
        key,
        value: value.take().unwrap(),
        expr: parse_expression(expr)?,
    })
}

fn parse_func_call(pair: Pair<Rule>) -> Result<FuncCall> {
    let mut pairs = pair.into_inner();
    let name = pairs.next().unwrap();
    let mut args = pairs.next().unwrap().into_inner();
    let builder = FuncCall::builder(name.as_str());

    args.try_fold(builder, |builder, pair| match pair.as_rule() {
        Rule::Variadic => Ok(builder.variadic(true)),
        _ => Ok(builder.arg(parse_expression(pair)?)),
    })
    .map(FuncCallBuilder::build)
}

fn parse_element_access_operator(pair: Pair<Rule>) -> Result<ElementAccessOperator> {
    let operator = match pair.as_rule() {
        Rule::AttrSplat => ElementAccessOperator::AttrSplat,
        Rule::FullSplat => ElementAccessOperator::FullSplat,
        Rule::GetAttr => ElementAccessOperator::GetAttr(parse_ident(inner(pair)).into()),
        Rule::Index => {
            let pair = inner(pair);

            match pair.as_rule() {
                Rule::LegacyIndex => {
                    ElementAccessOperator::LegacyIndex(parse_primitive::<u64>(pair))
                }
                _ => ElementAccessOperator::Index(parse_expression(pair)?),
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
    let raw = pair.as_str();

    // @FIXME(mohmann): according to the HCL spec, any expression is a valid object key. Fixing
    // this requires some breaking changes to the Object and ObjectKey types though.
    match pair.as_rule() {
        Rule::Identifier => Ok(ObjectKey::identifier(parse_ident(pair))),
        Rule::ExprTerm => match parse_expr_term(pair)? {
            Expression::String(s) => Ok(ObjectKey::String(s)),
            _ => Ok(ObjectKey::RawExpression(raw_expression(raw))),
        },
        _ => Ok(ObjectKey::RawExpression(raw_expression(raw))),
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

fn parse_ident(pair: Pair<Rule>) -> String {
    pair.as_str().to_owned()
}

fn raw_expression(raw: &str) -> RawExpression {
    RawExpression::new(raw.trim_end())
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
        delimiter: Identifier::new(delimiter.as_str()),
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
