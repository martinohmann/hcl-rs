use super::pratt::{parse_binary_op, BinaryOpToken};
use crate::expr::{BinaryOperator, Conditional, Expression, Traversal, TraversalOperator};
use crate::structure::{Body, Structure};
use crate::{Decorate, Decorated, RawString, SetSpan};
use fnv::FnvHashSet;
use std::ops::Range;

#[derive(Debug, Default)]
pub(super) struct BodyParseState<'a> {
    attribute_keys: FnvHashSet<&'a str>,
    current: Option<Structure>,
    structures: Vec<Structure>,
    ws: Option<Range<usize>>,
    eof: bool,
}

impl<'a> BodyParseState<'a> {
    pub(super) fn is_redefined(&mut self, key: &'a str) -> bool {
        !self.attribute_keys.insert(key)
    }

    pub(super) fn on_ws(&mut self, span: Range<usize>) {
        if let Some(existing) = &self.ws {
            self.ws = Some(existing.start..span.end);
        } else {
            self.ws = Some(span);
        }
    }

    pub(super) fn on_structure(&mut self, mut structure: Structure) {
        if let Some(prefix) = self.ws.take() {
            structure
                .decor_mut()
                .set_prefix(RawString::from_span(prefix));
        }

        self.current = Some(structure);
    }

    pub(super) fn on_line_ending(&mut self) {
        let mut current = self.current.take().unwrap();

        if let Some(suffix) = self.ws.take() {
            current.decor_mut().set_suffix(RawString::from_span(suffix));
        }

        self.structures.push(current);
    }

    pub(super) fn on_eof(&mut self) {
        self.on_line_ending();
        self.eof = true;
    }

    pub(super) fn into_body(self) -> Body {
        let mut body = Body::from_vec_unchecked(self.structures);
        body.set_prefer_omit_trailing_newline(self.eof);
        body
    }
}

#[derive(Debug, Default)]
pub(super) struct ExprParseState {
    current: Option<Expression>,
    ws: Option<Range<usize>>,
    allow_newlines: bool,
}

impl ExprParseState {
    pub(super) fn on_ws(&mut self, span: Range<usize>) {
        if let Some(existing) = &self.ws {
            self.ws = Some(existing.start..span.end);
        } else {
            self.ws = Some(span);
        }
    }

    pub(super) fn on_span(&mut self, span: Range<usize>) {
        if let Some(expr) = &mut self.current {
            expr.set_span(span);
        }
    }

    pub(super) fn on_expr_term(&mut self, expr: impl Into<Expression>) {
        let mut expr = expr.into();

        if let Some(prefix) = self.ws.take() {
            expr.decor_mut().set_prefix(RawString::from_span(prefix));
        }

        self.current = Some(expr);
    }

    pub(super) fn on_traversal(&mut self, operators: Vec<Decorated<TraversalOperator>>) {
        let mut expr_term = self.current.take().unwrap();

        if let Some(suffix) = self.ws.take() {
            expr_term
                .decor_mut()
                .set_suffix(RawString::from_span(suffix));
        }

        let traversal = Traversal::new(expr_term, operators);
        let expr = Expression::Traversal(Box::new(traversal));
        self.current = Some(expr);
    }

    pub(super) fn on_conditional(&mut self, true_expr: Expression, false_expr: Expression) {
        let mut cond_expr = self.current.take().unwrap();

        if let Some(suffix) = self.ws.take() {
            cond_expr
                .decor_mut()
                .set_suffix(RawString::from_span(suffix));
        }

        let conditional = Conditional::new(cond_expr, true_expr, false_expr);
        let expr = Expression::Conditional(Box::new(conditional));
        self.current = Some(expr);
    }

    pub(super) fn on_binary_ops(&mut self, ops: Vec<(Decorated<BinaryOperator>, Expression)>) {
        let mut lhs_expr = self.current.take().unwrap();

        if let Some(suffix) = self.ws.take() {
            lhs_expr
                .decor_mut()
                .set_suffix(RawString::from_span(suffix));
        }

        let mut tokens = Vec::with_capacity(ops.len() * 2 + 1);
        tokens.push(BinaryOpToken::Expression(lhs_expr));

        for (operator, expr) in ops {
            tokens.push(BinaryOpToken::Operator(operator));
            tokens.push(BinaryOpToken::Expression(expr));
        }

        self.current = Some(parse_binary_op(tokens));
    }

    pub(super) fn allow_newlines(&mut self) {
        self.allow_newlines = true;
    }

    pub(super) fn newlines_allowed(&self) -> bool {
        self.allow_newlines
    }

    pub(super) fn into_expr(self) -> Expression {
        self.current.unwrap()
    }
}

impl Clone for ExprParseState {
    fn clone(&self) -> Self {
        ExprParseState {
            current: None,
            ws: None,
            allow_newlines: self.allow_newlines,
        }
    }
}
