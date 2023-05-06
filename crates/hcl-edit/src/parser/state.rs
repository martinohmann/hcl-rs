use crate::{
    expr::{
        BinaryOp, BinaryOperator, Conditional, Expression, Traversal, TraversalOperator, UnaryOp,
        UnaryOperator,
    },
    repr::{Decorate, Decorated, SetSpan, Spanned},
    structure::{Body, Structure},
    RawString,
};
use fnv::FnvHashSet;
use std::ops::Range;

#[derive(Debug, Default)]
pub(super) struct BodyParseState<'a> {
    attribute_keys: FnvHashSet<&'a str>,
    current: Option<Structure>,
    structures: Vec<Structure>,
    ws: Option<Range<usize>>,
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

    pub(super) fn into_body(self) -> Body {
        Body::from(self.structures)
    }
}

#[derive(Debug, Default)]
pub(super) struct ExprParseState {
    unary: Option<Spanned<UnaryOperator>>,
    current: Option<Expression>,
    ws: Option<Range<usize>>,
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
        if let Some(operator) = self.unary.take() {
            expr = Expression::UnaryOp(Box::new(UnaryOp::new(operator, expr)));
        }

        if let Some(prefix) = self.ws.take() {
            expr.decor_mut().set_prefix(RawString::from_span(prefix));
        }

        self.current = Some(expr);
    }

    pub(super) fn on_unary_op(&mut self, operator: Spanned<UnaryOperator>, ws_span: Range<usize>) {
        self.unary = Some(operator);
        self.on_ws(ws_span);
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

    pub(super) fn on_binary_op(&mut self, operator: Spanned<BinaryOperator>, rhs_expr: Expression) {
        let mut lhs_expr = self.current.take().unwrap();

        if let Some(suffix) = self.ws.take() {
            lhs_expr
                .decor_mut()
                .set_suffix(RawString::from_span(suffix));
        }

        let binary_op = BinaryOp::new(lhs_expr, operator, rhs_expr);
        let expr = Expression::BinaryOp(Box::new(binary_op));
        self.current = Some(expr);
    }

    pub(super) fn into_expr(self) -> Expression {
        self.current.unwrap()
    }
}
