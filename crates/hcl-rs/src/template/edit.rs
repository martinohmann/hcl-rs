use super::*;
use crate::edit::template;

impl From<template::Template> for Template {
    fn from(value: template::Template) -> Self {
        Template::from_iter(value)
    }
}

impl From<Template> for template::Template {
    fn from(value: Template) -> Self {
        template::Template::from_iter(value.elements)
    }
}

impl From<template::Element> for Element {
    fn from(value: template::Element) -> Self {
        match value {
            template::Element::Literal(literal) => Element::Literal(literal.value_into()),
            template::Element::Directive(directive) => {
                Element::Directive(Box::new((*directive).into()))
            }
            template::Element::Interpolation(interp) => Element::Interpolation(interp.into()),
        }
    }
}

impl From<Element> for template::Element {
    fn from(value: Element) -> Self {
        match value {
            Element::Literal(literal) => template::Element::Literal(literal.into()),
            Element::Directive(directive) => {
                template::Element::Directive(Box::new((*directive).into()))
            }
            Element::Interpolation(interp) => template::Element::Interpolation(interp.into()),
        }
    }
}

impl From<template::Interpolation> for Interpolation {
    fn from(value: template::Interpolation) -> Self {
        Interpolation {
            expr: value.expr.into(),
            strip: value.strip,
        }
    }
}

impl From<Interpolation> for template::Interpolation {
    fn from(value: Interpolation) -> Self {
        let mut interp = template::Interpolation::new(value.expr);
        interp.strip = value.strip;
        interp
    }
}

impl From<template::Directive> for Directive {
    fn from(value: template::Directive) -> Self {
        match value {
            template::Directive::If(directive) => Directive::If(directive.into()),
            template::Directive::For(directive) => Directive::For(directive.into()),
        }
    }
}

impl From<Directive> for template::Directive {
    fn from(value: Directive) -> Self {
        match value {
            Directive::If(directive) => template::Directive::If(directive.into()),
            Directive::For(directive) => template::Directive::For(directive.into()),
        }
    }
}

impl From<template::IfDirective> for IfDirective {
    fn from(value: template::IfDirective) -> Self {
        let else_strip = value
            .else_expr
            .as_ref()
            .map(|expr| expr.strip)
            .unwrap_or_default();

        IfDirective {
            cond_expr: value.if_expr.cond_expr.into(),
            true_template: value.if_expr.template.into(),
            false_template: value.else_expr.map(|expr| expr.template.into()),
            if_strip: value.if_expr.strip,
            else_strip,
            endif_strip: value.endif_expr.strip,
        }
    }
}

impl From<IfDirective> for template::IfDirective {
    fn from(value: IfDirective) -> Self {
        let mut if_expr =
            template::IfTemplateExpr::new(value.cond_expr, value.true_template.into());
        if_expr.strip = value.if_strip;

        let else_expr = value.false_template.map(|template| {
            let mut else_expr = template::ElseTemplateExpr::new(template.into());
            else_expr.strip = value.else_strip;
            else_expr
        });

        let mut endif_expr = template::EndifTemplateExpr::new();
        endif_expr.strip = value.endif_strip;

        template::IfDirective::new(if_expr, else_expr, endif_expr)
    }
}

impl From<template::ForDirective> for ForDirective {
    fn from(value: template::ForDirective) -> Self {
        let for_expr = value.for_expr;
        let endfor_expr = value.endfor_expr;

        ForDirective {
            key_var: for_expr.key_var.map(Into::into),
            value_var: for_expr.value_var.into(),
            collection_expr: for_expr.collection_expr.into(),
            template: for_expr.template.into(),
            for_strip: for_expr.strip,
            endfor_strip: endfor_expr.strip,
        }
    }
}

impl From<ForDirective> for template::ForDirective {
    fn from(value: ForDirective) -> Self {
        let mut for_expr = template::ForTemplateExpr::new(
            value.key_var,
            value.value_var,
            value.collection_expr,
            value.template.into(),
        );
        for_expr.strip = value.for_strip;

        let mut endfor_expr = template::EndforTemplateExpr::new();
        endfor_expr.strip = value.endfor_strip;

        template::ForDirective::new(for_expr, endfor_expr)
    }
}
