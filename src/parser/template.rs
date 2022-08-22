use super::*;
use crate::template::{Element, Template};

pub fn parse_template(input: &str) -> Result<Template> {
    let pair = HclParser::parse(Rule::HclTemplate, input)?.next().unwrap();
    parse_template_elements(pair).map(|elements| Template { elements })
}

fn parse_template_elements(pair: Pair<Rule>) -> Result<Vec<Element>> {
    pair.into_inner().map(parse_template_element).collect()
}

fn parse_template_element(pair: Pair<Rule>) -> Result<Element> {
    match pair.as_rule() {
        Rule::TemplateLiteral => Ok(Element::Literal(parse_string(pair)?)),
        Rule::TemplateInterpolation => todo!(),
        Rule::TemplateDirective => todo!(),
        rule => unexpected_rule(rule),
    }
}
