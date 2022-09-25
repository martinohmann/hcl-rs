use super::*;
use crate::template::{ForExpr, *};

pub fn parse(input: &str) -> Result<Template> {
    let pair = HclParser::parse(Rule::HclTemplate, input)?.next().unwrap();
    parse_template(pair)
}

fn parse_template(pair: Pair<Rule>) -> Result<Template> {
    pair.into_inner().map(parse_element).collect()
}

fn parse_element(pair: Pair<Rule>) -> Result<Element> {
    match pair.as_rule() {
        Rule::TemplateLiteral => Ok(Element::Literal(pair.as_str().to_owned())),
        Rule::TemplateInterpolation => parse_interpolation(pair).map(Element::Interpolation),
        Rule::TemplateDirective => parse_directive(pair).map(Element::Directive),
        rule => unexpected_rule(rule),
    }
}

fn parse_interpolation(pair: Pair<Rule>) -> Result<Interpolation> {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let expr = pairs.next().unwrap();
    let end = pairs.next().unwrap();

    Ok(Interpolation {
        expr: parse_expression(expr)?,
        strip: parse_strip_mode(start, end),
    })
}

fn parse_directive(pair: Pair<Rule>) -> Result<Directive> {
    let pair = inner(pair);

    match pair.as_rule() {
        Rule::TemplateIf => parse_if_directive(pair).map(Directive::If),
        Rule::TemplateFor => parse_for_directive(pair).map(Directive::For),
        rule => unexpected_rule(rule),
    }
}

fn parse_if_directive(pair: Pair<Rule>) -> Result<IfDirective> {
    let mut pairs = pair.into_inner();
    let expr = pairs.next().unwrap();
    let if_expr = parse_if_expr(expr)?;

    let mut expr = pairs.next().unwrap();

    // Else branch is optional.
    let else_expr = match expr.as_rule() {
        Rule::TemplateElseExpr => {
            let else_expr = parse_else_expr(expr)?;
            expr = pairs.next().unwrap();
            Some(else_expr)
        }
        Rule::TemplateEndIfExpr => None,
        rule => unexpected_rule(rule),
    };

    Ok(IfDirective {
        if_expr,
        else_expr,
        strip: parse_end_expr_strip_mode(expr),
    })
}

fn parse_if_expr(pair: Pair<Rule>) -> Result<IfExpr> {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let expr = pairs.next().unwrap();
    let end = pairs.next().unwrap();
    let template = pairs.next().unwrap();

    Ok(IfExpr {
        cond_expr: parse_expression(expr)?,
        strip: parse_strip_mode(start, end),
        template: parse_template(template)?,
    })
}

fn parse_else_expr(pair: Pair<Rule>) -> Result<ElseExpr> {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let end = pairs.next().unwrap();
    let template = pairs.next().unwrap();

    Ok(ElseExpr {
        strip: parse_strip_mode(start, end),
        template: parse_template(template)?,
    })
}

fn parse_for_directive(pair: Pair<Rule>) -> Result<ForDirective> {
    let mut pairs = pair.into_inner();
    let for_expr = pairs.next().unwrap();
    let endfor_expr = pairs.next().unwrap();

    Ok(ForDirective {
        for_expr: parse_for_expr(for_expr)?,
        strip: parse_end_expr_strip_mode(endfor_expr),
    })
}

fn parse_for_expr(pair: Pair<Rule>) -> Result<ForExpr> {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let mut value_var = Some(Identifier::new(pairs.next().unwrap().as_str()));
    let mut expr = pairs.next().unwrap();

    // If there are two identifiers, the first one is the key and the second one the value.
    let key_var = match expr.as_rule() {
        Rule::Identifier => {
            let key_var = value_var.replace(Identifier::new(expr.as_str()));
            expr = pairs.next().unwrap();
            key_var
        }
        _ => None,
    };

    let end = pairs.next().unwrap();
    let template = pairs.next().unwrap();

    Ok(ForExpr {
        key_var,
        value_var: value_var.take().unwrap(),
        collection_expr: parse_expression(expr)?,
        template: parse_template(template)?,
        strip: parse_strip_mode(start, end),
    })
}

fn parse_end_expr_strip_mode(pair: Pair<Rule>) -> StripMode {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let end = pairs.next().unwrap();

    parse_strip_mode(start, end)
}

fn parse_strip_mode(start: Pair<Rule>, end: Pair<Rule>) -> StripMode {
    let strip_start = match start.as_rule() {
        Rule::TemplateIExprStartStrip | Rule::TemplateDExprStartStrip => true,
        Rule::TemplateIExprStartNormal | Rule::TemplateDExprStartNormal => false,
        rule => unexpected_rule(rule),
    };

    let strip_end = match end.as_rule() {
        Rule::TemplateExprEndStrip => true,
        Rule::TemplateExprEndNormal => false,
        rule => unexpected_rule(rule),
    };

    match (strip_start, strip_end) {
        (true, true) => StripMode::Both,
        (true, false) => StripMode::Start,
        (false, true) => StripMode::End,
        (false, false) => StripMode::None,
    }
}
