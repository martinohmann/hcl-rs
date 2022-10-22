use super::*;
use crate::template::*;

pub fn parse(input: &str) -> Result<Template> {
    let pair = HclParser::parse(Rule::HclTemplate, input)?.next().unwrap();
    parse_template(inner(pair))
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
    let if_expr = parse_if_expr(pairs.next().unwrap())?;
    let mut expr = pairs.next().unwrap();

    // Else branch is optional.
    let (false_template, else_strip) = match expr.as_rule() {
        Rule::TemplateElseExpr => {
            let else_expr = parse_else_expr(expr)?;
            expr = pairs.next().unwrap();
            (Some(else_expr.false_template), else_expr.else_strip)
        }
        Rule::TemplateEndIfExpr => (None, StripMode::default()),
        rule => unexpected_rule(rule),
    };

    Ok(IfDirective {
        cond_expr: if_expr.cond_expr,
        true_template: if_expr.true_template,
        false_template,
        if_strip: if_expr.if_strip,
        else_strip,
        endif_strip: parse_end_expr_strip_mode(expr),
    })
}

struct IfExpr {
    cond_expr: Expression,
    true_template: Template,
    if_strip: StripMode,
}

fn parse_if_expr(pair: Pair<Rule>) -> Result<IfExpr> {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let expr = pairs.next().unwrap();
    let end = pairs.next().unwrap();
    let template = pairs.next().unwrap();

    Ok(IfExpr {
        cond_expr: parse_expression(expr)?,
        if_strip: parse_strip_mode(start, end),
        true_template: parse_template(template)?,
    })
}

struct ElseExpr {
    false_template: Template,
    else_strip: StripMode,
}

fn parse_else_expr(pair: Pair<Rule>) -> Result<ElseExpr> {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let end = pairs.next().unwrap();
    let template = pairs.next().unwrap();

    Ok(ElseExpr {
        else_strip: parse_strip_mode(start, end),
        false_template: parse_template(template)?,
    })
}

fn parse_for_directive(pair: Pair<Rule>) -> Result<ForDirective> {
    let mut pairs = pair.into_inner();
    let for_expr = parse_for_expr(pairs.next().unwrap())?;
    let endfor_expr = pairs.next().unwrap();

    Ok(ForDirective {
        key_var: for_expr.key_var,
        value_var: for_expr.value_var,
        collection_expr: for_expr.collection_expr,
        template: for_expr.template,
        for_strip: for_expr.for_strip,
        endfor_strip: parse_end_expr_strip_mode(endfor_expr),
    })
}

struct ForExpr {
    key_var: Option<Identifier>,
    value_var: Identifier,
    collection_expr: Expression,
    template: Template,
    for_strip: StripMode,
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
        for_strip: parse_strip_mode(start, end),
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

    StripMode::from((strip_start, strip_end))
}
