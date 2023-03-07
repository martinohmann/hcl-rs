use super::*;
use crate::template::{
    Directive, Element, ForDirective, IfDirective, Interpolation, StripMode, Template,
};

pub fn template(pair: Pair<Rule>) -> Result<Template> {
    pair.into_inner().map(element).collect()
}

fn element(pair: Pair<Rule>) -> Result<Element> {
    match pair.as_rule() {
        Rule::TemplateLiteral => Ok(Element::Literal(string(pair))),
        Rule::TemplateInterpolation => interpolation(pair).map(Element::Interpolation),
        Rule::TemplateDirective => directive(inner(pair)).map(Element::Directive),
        rule => unexpected_rule(rule),
    }
}

fn interpolation(pair: Pair<Rule>) -> Result<Interpolation> {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let expr = pairs.next().unwrap();
    let end = pairs.next().unwrap();

    Ok(Interpolation {
        expr: expression(expr)?,
        strip: strip_mode(start, end),
    })
}

fn directive(pair: Pair<Rule>) -> Result<Directive> {
    match pair.as_rule() {
        Rule::TemplateIf => if_directive(pair).map(Directive::If),
        Rule::TemplateFor => for_directive(pair).map(Directive::For),
        rule => unexpected_rule(rule),
    }
}

fn if_directive(pair: Pair<Rule>) -> Result<IfDirective> {
    let mut pairs = pair.into_inner();
    let if_expr = if_expr(pairs.next().unwrap())?;
    let true_template = template(pairs.next().unwrap())?;
    let mut expr = pairs.next().unwrap();

    // Else branch is optional.
    let (false_template, else_strip) = match expr.as_rule() {
        Rule::TemplateElseExpr => {
            let else_strip = else_expr_strip_mode(expr);
            let false_template = template(pairs.next().unwrap())?;
            expr = pairs.next().unwrap();
            (Some(false_template), else_strip)
        }
        Rule::TemplateEndIfExpr => (None, StripMode::default()),
        rule => unexpected_rule(rule),
    };

    Ok(IfDirective {
        cond_expr: if_expr.cond_expr,
        true_template,
        false_template,
        if_strip: if_expr.if_strip,
        else_strip,
        endif_strip: end_expr_strip_mode(expr),
    })
}

struct IfExpr {
    cond_expr: Expression,
    if_strip: StripMode,
}

fn if_expr(pair: Pair<Rule>) -> Result<IfExpr> {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let cond_expr = expression(pairs.next().unwrap())?;
    let end = pairs.next().unwrap();

    Ok(IfExpr {
        cond_expr,
        if_strip: strip_mode(start, end),
    })
}

fn else_expr_strip_mode(pair: Pair<Rule>) -> StripMode {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let end = pairs.next().unwrap();
    strip_mode(start, end)
}

fn for_directive(pair: Pair<Rule>) -> Result<ForDirective> {
    let mut pairs = pair.into_inner();
    let for_expr = for_expr(pairs.next().unwrap())?;
    let template = template(pairs.next().unwrap())?;
    let endfor_strip = end_expr_strip_mode(pairs.next().unwrap());

    Ok(ForDirective {
        key_var: for_expr.key_var,
        value_var: for_expr.value_var,
        collection_expr: for_expr.collection_expr,
        template,
        for_strip: for_expr.for_strip,
        endfor_strip,
    })
}

struct ForExpr {
    key_var: Option<Identifier>,
    value_var: Identifier,
    collection_expr: Expression,
    for_strip: StripMode,
}

fn for_expr(pair: Pair<Rule>) -> Result<ForExpr> {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let mut value_var = Some(ident(pairs.next().unwrap()));
    let mut expr = pairs.next().unwrap();

    // If there are two identifiers, the first one is the key and the second one the value.
    let key_var = match expr.as_rule() {
        Rule::Identifier => {
            let key_var = value_var.replace(ident(expr));
            expr = pairs.next().unwrap();
            key_var
        }
        _ => None,
    };

    let end = pairs.next().unwrap();

    Ok(ForExpr {
        key_var,
        value_var: value_var.take().unwrap(),
        collection_expr: expression(expr)?,
        for_strip: strip_mode(start, end),
    })
}

fn end_expr_strip_mode(pair: Pair<Rule>) -> StripMode {
    let mut pairs = pair.into_inner();
    let start = pairs.next().unwrap();
    let end = pairs.next().unwrap();
    strip_mode(start, end)
}

fn strip_mode(start: Pair<Rule>, end: Pair<Rule>) -> StripMode {
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
