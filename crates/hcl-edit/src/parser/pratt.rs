use crate::expr::{BinaryOp, BinaryOperator, Expression};
use crate::{Decorated, SetSpan, Span};
use pratt::{Affix, Associativity, NoError, PrattParser, Precedence, Result};

/// Valid tokens in binary operations.
#[derive(Debug)]
pub(super) enum BinaryOpToken {
    Operator(Decorated<BinaryOperator>),
    Expression(Expression),
}

#[inline]
pub(super) fn parse_binary_op(tokens: Vec<BinaryOpToken>) -> Expression {
    debug_assert!(!tokens.is_empty());
    let mut tokens = tokens.into_iter().peekable();
    let expr = BinaryOpParser
        .parse_peekable(&mut tokens)
        .expect("BinaryOpParser cannot fail");
    debug_assert!(
        tokens.peek().is_none(),
        "unconsumed binary operation tokens"
    );
    expr
}

/// A Pratt parser for operator precedence resolution.
struct BinaryOpParser;

impl<I> PrattParser<I> for BinaryOpParser
where
    I: Iterator<Item = BinaryOpToken>,
{
    type Error = NoError;
    type Input = BinaryOpToken;
    type Output = Expression;

    fn query(&mut self, token: &BinaryOpToken) -> Result<Affix> {
        match token {
            BinaryOpToken::Operator(operator) => {
                let precedence = u32::from(operator.precedence());
                // All binary operators within the same precedence level have left-to-right
                // associativity, including comparison operators.
                Ok(Affix::Infix(Precedence(precedence), Associativity::Left))
            }
            BinaryOpToken::Expression(_) => Ok(Affix::Nilfix),
        }
    }

    fn primary(&mut self, token: BinaryOpToken) -> Result<Expression> {
        match token {
            BinaryOpToken::Expression(expr) => Ok(expr),
            BinaryOpToken::Operator(_) => unreachable!(),
        }
    }

    fn infix(
        &mut self,
        lhs: Expression,
        token: BinaryOpToken,
        rhs: Expression,
    ) -> Result<Expression> {
        match token {
            BinaryOpToken::Operator(operator) => match (lhs.span(), rhs.span()) {
                (Some(lhs_span), Some(rhs_span)) => {
                    let mut expr = Expression::from(BinaryOp::new(lhs, operator, rhs));
                    expr.set_span(lhs_span.start..rhs_span.end);
                    Ok(expr)
                }
                (_, _) => Ok(Expression::from(BinaryOp::new(lhs, operator, rhs))),
            },
            BinaryOpToken::Expression(_) => unreachable!(),
        }
    }

    fn prefix(&mut self, _token: BinaryOpToken, _rhs: Expression) -> Result<Expression> {
        unreachable!()
    }

    fn postfix(&mut self, _lhs: Expression, _token: BinaryOpToken) -> Result<Expression> {
        unreachable!()
    }
}
