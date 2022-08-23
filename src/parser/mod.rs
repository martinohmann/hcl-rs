mod template;
#[cfg(test)]
mod tests;
mod unescape;

use crate::{
    structure::Identifier, Attribute, Block, BlockLabel, Body, Expression, Heredoc,
    HeredocStripMode, Number, Object, ObjectKey, RawExpression, Result, Structure, TemplateExpr,
};
use pest::{
    iterators::{Pair, Pairs},
    Parser as ParserTrait,
};
use pest_derive::Parser;
use std::str::FromStr;
pub use template::parse as parse_template;
use unescape::unescape;

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
    let raw = pair.as_str();
    let mut pairs = pair.into_inner();
    let pair = pairs.next().unwrap();

    if pairs.peek().is_some() {
        // @TODO(mohmann): ExprTerm is followed by an traversal like Splat, GetAttr or Index.
        // For now we treat these as raw expressions and not further process these.
        Ok(Expression::Raw(raw_expression(raw)))
    } else {
        match pair.as_rule() {
            Rule::BooleanLit => Ok(Expression::Bool(parse_primitive(pair))),
            Rule::Float => Ok(Expression::Number(parse_primitive::<f64>(pair).into())),
            Rule::Int => Ok(Expression::Number(parse_primitive::<i64>(pair).into())),
            Rule::NullLit => Ok(Expression::Null),
            Rule::StringLit => parse_string(inner(pair)).map(Expression::String),
            Rule::TemplateExpr => parse_template_expr(inner(pair)).map(Into::into),
            Rule::Tuple => parse_expressions(pair).map(Expression::Array),
            Rule::Object => parse_object(pair).map(Expression::Object),
            // @TODO(mohmann): Process ForExpr, VariableExpr etc.
            _ => Ok(Expression::Raw(raw_expression(pair.as_str()))),
        }
    }
}

fn parse_template_expr(pair: Pair<Rule>) -> Result<TemplateExpr> {
    match pair.as_rule() {
        Rule::QuotedStringTemplate => parse_string(inner(pair)).map(TemplateExpr::QuotedString),
        Rule::HeredocTemplate => parse_heredoc(pair).map(TemplateExpr::Heredoc),
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
    unescape(pair.as_str())
}

fn parse_ident(pair: Pair<Rule>) -> String {
    pair.as_str().to_owned()
}

fn raw_expression(raw: &str) -> RawExpression {
    RawExpression::new(raw.trim_end())
}

fn parse_heredoc(pair: Pair<Rule>) -> Result<Heredoc> {
    let mut pairs = pair.into_inner();
    let intro = pairs.next().unwrap();
    let delimiter = pairs.next().unwrap();
    let template = pairs.next().unwrap();

    let strip = match intro.as_rule() {
        Rule::HeredocIntroNormal => HeredocStripMode::None,
        Rule::HeredocIntroIndent => HeredocStripMode::Indent,
        rule => unexpected_rule(rule),
    };

    Ok(Heredoc {
        strip,
        delimiter: Identifier::new(delimiter.as_str()),
        template: template.as_str().to_owned(),
    })
}

// String dedent implementation which does not distinguish between spaces, tabs or unicode
// whitespace but simply treats all of them as "one unit of whitespace".
//
// This is how the original HCL spec seems to handle it based on the original specsuite although it
// is not formally defined. E.g. ' ' (space) and '\u{2003}' (unicode "em-space") are treated as one
// unit of whitespace even though the former is 1 byte and the latter is 3 bytes long.
pub(crate) fn dedent_string(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    let mut leading_ws = usize::MAX;
    let mut non_empty_lines = 0;

    // Find the minimum number of possible leading units of whitespace that can be be stripped off
    // of each non-empty line.
    for line in s.lines().filter(|line| !line.is_empty()) {
        let line_leading_ws = line.chars().take_while(|ch| ch.is_whitespace()).count();

        if line_leading_ws == 0 {
            // Fast path: no dedent needed if we encounter a non-empty line which starts with a
            // non-whitespace character.
            return s.to_string();
        }

        leading_ws = leading_ws.min(line_leading_ws);
        non_empty_lines += 1;
    }

    // Strip the determined amount of leading whitespace off of each line.
    let mut dedented = String::with_capacity(s.len() - leading_ws * non_empty_lines);

    for line in s.lines() {
        if !line.is_empty() {
            dedented.extend(line.chars().skip(leading_ws));
        }

        dedented.push('\n');
    }

    if dedented.ends_with('\n') && !s.ends_with('\n') {
        let new_len = dedented.len() - 1;
        dedented.truncate(new_len);
    }

    dedented
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
