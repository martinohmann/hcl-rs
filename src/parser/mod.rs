mod template;
#[cfg(test)]
mod tests;

use crate::{
    structure::Identifier, util::unescape, Attribute, Block, BlockLabel, Body,
    ElementAccessOperator, Expression, Heredoc, HeredocStripMode, Number, Object, ObjectKey,
    RawExpression, Result, Structure, TemplateExpr,
};
use pest::{
    iterators::{Pair, Pairs},
    Parser as ParserTrait,
};
use pest_derive::Parser;
use std::str::FromStr;
pub use template::parse as parse_template;

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
    // @TODO(mohmann): Process Conditional and Operation at one point. This will require a
    // PrecClimber to get precedence right though.
    match pair.as_rule() {
        Rule::ExprTerm => parse_expr_term(pair),
        Rule::Operation => {
            let raw = pair.as_str();
            // We only fully parse unary operations representing negative numbers for now.
            // Everything else will be represented as raw expressions until parsing operations is
            // fully implemented.
            match parse_negative_number(inner(pair))? {
                Some(num) => Ok(Expression::Number(num)),
                None => Ok(Expression::Raw(raw_expression(raw))),
            }
        }
        Rule::Conditional => Ok(Expression::Raw(raw_expression(pair.as_str()))),
        rule => unexpected_rule(rule),
    }
}

fn parse_negative_number(pair: Pair<Rule>) -> Result<Option<Number>> {
    if let Rule::UnaryOp = pair.as_rule() {
        let mut pairs = pair.into_inner();
        let operator = pairs.next().unwrap();
        let expr = pairs.next().unwrap();

        if let ("-", Rule::ExprTerm) = (operator.as_str(), expr.as_rule()) {
            if let Expression::Number(num) = parse_expr_term(expr)? {
                return Ok(Some(-num));
            }
        }
    }

    Ok(None)
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
        // @TODO(mohmann): Process ForExpr etc.
        _ => Expression::Raw(raw_expression(pair.as_str())),
    };

    pairs.try_fold(expr, |expr, pair| {
        Ok(expr.element(parse_element_access_operator(pair)?))
    })
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
