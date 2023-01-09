mod expr;
mod structure;
mod template;
#[cfg(test)]
mod tests;
mod v2;

pub use self::v2::parse;
use self::{expr::expression, structure::body, template::template};
use crate::{
    expr::Expression, structure::Body, template::Template, util::unescape, Identifier, Number,
    Result,
};
use pest::{
    iterators::{Pair, Pairs},
    Parser as _,
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
///
/// # Errors
///
/// This function fails with an error if the `input` cannot be parsed as HCL.
pub fn parse2(input: &str) -> Result<Body> {
    let pair = HclParser::parse(Rule::Hcl, input)?.next().unwrap();
    body(pair)
}

pub fn parse_template(input: &str) -> Result<Template> {
    let pair = HclParser::parse(Rule::HclTemplate, input)?.next().unwrap();
    template(inner(pair))
}

fn string(pair: Pair<Rule>) -> String {
    pair.as_str().to_owned()
}

fn unescape_string(pair: Pair<Rule>) -> Result<String> {
    unescape(pair.as_str()).map(|c| c.to_string())
}

fn ident(pair: Pair<Rule>) -> Identifier {
    Identifier::unchecked(pair.as_str())
}

fn from_str<T>(pair: Pair<Rule>) -> T
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    pair.as_str().parse::<T>().unwrap()
}

fn inner(pair: Pair<Rule>) -> Pair<Rule> {
    pair.into_inner().next().unwrap()
}

#[track_caller]
fn unexpected_rule(rule: Rule) -> ! {
    panic!("unexpected rule: {rule:?}")
}
