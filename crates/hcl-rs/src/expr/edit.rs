use super::*;
use crate::edit::{expr, template, Decorated, Ident};

impl From<expr::Expression> for Expression {
    fn from(value: expr::Expression) -> Self {
        match value {
            expr::Expression::Null(_) => Expression::Null,
            expr::Expression::Bool(b) => b.value_into(),
            expr::Expression::Array(array) => Expression::from_iter(array),
            expr::Expression::Number(num) => num.value_into(),
            expr::Expression::String(string) => string.value_into(),
            expr::Expression::Object(object) => Expression::from_iter(object),
            expr::Expression::UnaryOp(unary) => UnaryOp::from(*unary).into(),
            expr::Expression::BinaryOp(binary) => BinaryOp::from(*binary).into(),
            expr::Expression::ForExpr(for_expr) => ForExpr::from(*for_expr).into(),
            expr::Expression::StringTemplate(template) => TemplateExpr::from(template).into(),
            expr::Expression::HeredocTemplate(template) => Heredoc::from(*template).into(),
            expr::Expression::Variable(variable) => Expression::Variable(variable.value_into()),
            expr::Expression::FuncCall(func_call) => FuncCall::from(*func_call).into(),
            expr::Expression::Traversal(traversal) => Traversal::from(*traversal).into(),
            expr::Expression::Parenthesis(parens) => {
                Expression::Parenthesis(Box::new(parens.into_inner().into()))
            }
            expr::Expression::Conditional(cond) => Conditional::from(*cond).into(),
        }
    }
}

impl From<Expression> for expr::Expression {
    fn from(value: Expression) -> Self {
        match value {
            Expression::Null => expr::Expression::null(),
            Expression::Bool(b) => b.into(),
            Expression::Array(array) => expr::Expression::from_iter(array),
            Expression::Number(num) => num.into(),
            Expression::String(string) => string.into(),
            Expression::Object(object) => expr::Expression::from_iter(object),
            Expression::Operation(op) => match *op {
                Operation::Unary(unary) => expr::UnaryOp::from(unary).into(),
                Operation::Binary(binary) => expr::BinaryOp::from(binary).into(),
            },
            Expression::ForExpr(for_expr) => expr::ForExpr::from(*for_expr).into(),
            Expression::TemplateExpr(template) => match *template {
                TemplateExpr::Heredoc(heredoc) => template::HeredocTemplate::from(heredoc).into(),
                TemplateExpr::QuotedString(template) => {
                    template::StringTemplate::from(template_or_default(template)).into()
                }
            },
            Expression::Variable(variable) => {
                expr::Expression::Variable(variable.into_inner().into())
            }
            Expression::FuncCall(func_call) => expr::FuncCall::from(*func_call).into(),
            Expression::Traversal(traversal) => expr::Traversal::from(*traversal).into(),
            Expression::Parenthesis(parens) => expr::Parenthesis::new((*parens).into()).into(),
            Expression::Conditional(cond) => expr::Conditional::from(*cond).into(),
            Expression::Raw(raw) => {
                template::StringTemplate::from(template_or_default(raw.as_str())).into()
            }
        }
    }
}

impl From<expr::ObjectKey> for ObjectKey {
    fn from(value: expr::ObjectKey) -> Self {
        match value {
            expr::ObjectKey::Ident(ident) => ObjectKey::Identifier(ident.into()),
            expr::ObjectKey::Expression(expr) => ObjectKey::Expression(expr.into()),
        }
    }
}

impl From<ObjectKey> for expr::ObjectKey {
    fn from(value: ObjectKey) -> Self {
        match value {
            ObjectKey::Identifier(ident) => expr::ObjectKey::Ident(ident.into()),
            ObjectKey::Expression(expr) => expr::ObjectKey::Expression(expr.into()),
        }
    }
}

impl From<expr::ObjectValue> for Expression {
    fn from(value: expr::ObjectValue) -> Self {
        value.into_expr().into()
    }
}

impl From<Expression> for expr::ObjectValue {
    fn from(value: Expression) -> Self {
        expr::ObjectValue::new(value)
    }
}

impl From<expr::Conditional> for Conditional {
    fn from(value: expr::Conditional) -> Self {
        Conditional::new(value.cond_expr, value.true_expr, value.false_expr)
    }
}

impl From<Conditional> for expr::Conditional {
    fn from(value: Conditional) -> Self {
        expr::Conditional::new(value.cond_expr, value.true_expr, value.false_expr)
    }
}

impl From<expr::ForExpr> for ForExpr {
    fn from(value: expr::ForExpr) -> Self {
        let intro = value.intro;
        ForExpr {
            key_var: intro.key_var.map(Into::into),
            value_var: intro.value_var.into(),
            collection_expr: intro.collection_expr.into(),
            key_expr: value.key_expr.map(Into::into),
            value_expr: value.value_expr.into(),
            grouping: value.grouping,
            cond_expr: value.cond.map(|cond| cond.expr.into()),
        }
    }
}

impl From<ForExpr> for expr::ForExpr {
    fn from(value: ForExpr) -> Self {
        let mut intro = expr::ForIntro::new(value.value_var, value.collection_expr);
        intro.key_var = value.key_var.map(Into::into);

        let mut for_expr = expr::ForExpr::new(intro, value.value_expr);
        for_expr.key_expr = value.key_expr.map(Into::into);
        for_expr.grouping = value.grouping;
        for_expr.cond = value.cond_expr.map(expr::ForCond::new);
        for_expr
    }
}

impl From<expr::FuncName> for FuncName {
    fn from(value: expr::FuncName) -> Self {
        FuncName {
            namespace: value.namespace.into_iter().map(Into::into).collect(),
            name: value.name.into(),
        }
    }
}

impl From<FuncName> for expr::FuncName {
    fn from(value: FuncName) -> Self {
        expr::FuncName {
            namespace: value.namespace.into_iter().map(Into::into).collect(),
            name: value.name.into(),
        }
    }
}

impl From<expr::FuncCall> for FuncCall {
    fn from(value: expr::FuncCall) -> Self {
        let expand_final = value.args.expand_final();
        FuncCall {
            name: value.name.into(),
            args: value.args.into_iter().map(Into::into).collect(),
            expand_final,
        }
    }
}

impl From<FuncCall> for expr::FuncCall {
    fn from(value: FuncCall) -> Self {
        let mut args = expr::FuncArgs::from_iter(value.args);
        args.set_expand_final(value.expand_final);
        expr::FuncCall::new(value.name, args)
    }
}

impl From<expr::UnaryOp> for Operation {
    fn from(value: expr::UnaryOp) -> Self {
        Operation::Unary(value.into())
    }
}

impl From<expr::BinaryOp> for Operation {
    fn from(value: expr::BinaryOp) -> Self {
        Operation::Binary(value.into())
    }
}

impl From<expr::UnaryOp> for UnaryOp {
    fn from(value: expr::UnaryOp) -> Self {
        UnaryOp::new(value.operator.value_into(), value.expr)
    }
}

impl From<UnaryOp> for expr::UnaryOp {
    fn from(value: UnaryOp) -> Self {
        expr::UnaryOp::new(value.operator, value.expr)
    }
}

impl From<expr::BinaryOp> for BinaryOp {
    fn from(value: expr::BinaryOp) -> Self {
        BinaryOp::new(value.lhs_expr, value.operator.value_into(), value.rhs_expr)
    }
}

impl From<BinaryOp> for expr::BinaryOp {
    fn from(value: BinaryOp) -> Self {
        expr::BinaryOp::new(value.lhs_expr, value.operator, value.rhs_expr)
    }
}

impl From<template::StringTemplate> for TemplateExpr {
    fn from(value: template::StringTemplate) -> Self {
        TemplateExpr::QuotedString(template::Template::from(value).to_string())
    }
}

impl From<template::HeredocTemplate> for Heredoc {
    fn from(value: template::HeredocTemplate) -> Self {
        let strip = value
            .indent()
            .map_or(HeredocStripMode::None, |_| HeredocStripMode::Indent);

        Heredoc {
            delimiter: value.delimiter.into(),
            template: value.template.to_string(),
            strip,
        }
    }
}

impl From<Heredoc> for template::HeredocTemplate {
    fn from(value: Heredoc) -> Self {
        template::HeredocTemplate::new(value.delimiter.into(), template_or_default(value.template))
    }
}

impl From<expr::Traversal> for Traversal {
    fn from(value: expr::Traversal) -> Self {
        Traversal::new(value.expr, value.operators)
    }
}

impl From<Traversal> for expr::Traversal {
    fn from(value: Traversal) -> Self {
        expr::Traversal::new(
            value.expr,
            value.operators.into_iter().map(Into::into).collect(),
        )
    }
}

impl From<Decorated<expr::TraversalOperator>> for TraversalOperator {
    fn from(value: Decorated<expr::TraversalOperator>) -> Self {
        value.value_into()
    }
}

impl From<TraversalOperator> for Decorated<expr::TraversalOperator> {
    fn from(value: TraversalOperator) -> Self {
        Decorated::new(value.into())
    }
}

impl From<expr::TraversalOperator> for TraversalOperator {
    fn from(value: expr::TraversalOperator) -> Self {
        match value {
            expr::TraversalOperator::Index(index) => Expression::from(index).into(),
            expr::TraversalOperator::GetAttr(ident) => ident.into(),
            expr::TraversalOperator::AttrSplat(_) => TraversalOperator::AttrSplat,
            expr::TraversalOperator::FullSplat(_) => TraversalOperator::FullSplat,
            expr::TraversalOperator::LegacyIndex(index) => index.value_into(),
        }
    }
}

impl From<TraversalOperator> for expr::TraversalOperator {
    fn from(value: TraversalOperator) -> Self {
        match value {
            TraversalOperator::Index(index) => expr::TraversalOperator::Index(index.into()),
            TraversalOperator::GetAttr(ident) => expr::TraversalOperator::GetAttr(ident.into()),
            TraversalOperator::AttrSplat => expr::TraversalOperator::AttrSplat(expr::Splat.into()),
            TraversalOperator::FullSplat => expr::TraversalOperator::FullSplat(expr::Splat.into()),
            TraversalOperator::LegacyIndex(index) => {
                expr::TraversalOperator::LegacyIndex(index.into())
            }
        }
    }
}

impl From<Ident> for Variable {
    fn from(ident: Ident) -> Self {
        Identifier::from(ident).into()
    }
}

impl From<Decorated<Ident>> for Identifier {
    fn from(value: Decorated<Ident>) -> Self {
        value.value_into()
    }
}

impl From<Identifier> for Decorated<Ident> {
    fn from(value: Identifier) -> Self {
        Decorated::new(value.into())
    }
}

fn template_or_default<T: AsRef<str>>(s: T) -> template::Template {
    s.as_ref().parse().unwrap_or_default()
}
