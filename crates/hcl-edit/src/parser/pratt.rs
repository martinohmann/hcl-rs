use crate::expr::{BinaryOp, BinaryOperator, Expression};
use crate::Decorated;
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
    BinaryOpParser
        .parse(tokens.into_iter())
        .expect("BinaryOpParser cannot fail")
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
                let associativity = match **operator {
                    BinaryOperator::Eq
                    | BinaryOperator::NotEq
                    | BinaryOperator::LessEq
                    | BinaryOperator::GreaterEq
                    | BinaryOperator::Less
                    | BinaryOperator::Greater => Associativity::Neither,
                    BinaryOperator::Plus
                    | BinaryOperator::Minus
                    | BinaryOperator::Mul
                    | BinaryOperator::Div
                    | BinaryOperator::Mod
                    | BinaryOperator::And
                    | BinaryOperator::Or => Associativity::Left,
                };

                Ok(Affix::Infix(Precedence(precedence), associativity))
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
            BinaryOpToken::Operator(operator) => {
                Ok(Expression::from(BinaryOp::new(lhs, operator, rhs)))
            }
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
