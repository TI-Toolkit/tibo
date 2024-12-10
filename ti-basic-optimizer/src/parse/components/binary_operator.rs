use crate::parse::components::{ListIndexable, MatrixIndexable, Operand, Operator, OperatorKind};
use crate::parse::expression::Expression;
use crate::parse::Reconstruct;
use crate::Config;
use std::iter::once;
use titokens::{Token, Version};

#[derive(Clone, Debug)]
pub struct BinOp {
    pub kind: Token,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl OperatorKind for BinOp {
    fn recognize(token: Token) -> bool {
        matches!(
            token,
            Token::OneByte(0x3C) | // Or
            Token::OneByte(0x3D) | // Xor
            Token::OneByte(0x40) | // And

            Token::OneByte(0x6A) | // Eq
            Token::OneByte(0x6B) | // Lt
            Token::OneByte(0x6C) | // Gt
            Token::OneByte(0x6D) | // Ne
            Token::OneByte(0x6E) | // Le
            Token::OneByte(0x6F) | // Ge

            Token::OneByte(0x70) | // Add
            Token::OneByte(0x71) | // Sub
            Token::OneByte(0x82) | // Mul
            Token::OneByte(0x83) | // Div

            Token::OneByte(0x94) | // Npr
            Token::OneByte(0x95) | // Ncr

            Token::OneByte(0xF0) | // Power
            Token::OneByte(0xF1) //   XRoot
        )
    }
}

impl BinOp {
    /// If `left` and `right` are flipped, which operator would produce the same result?
    pub fn opposite(&self) -> Option<Token> {
        match self.kind {
            // + * or xor and = !=
            Token::OneByte(x) if matches!(x, 0x70 | 0x82 | 0x3C | 0x3D | 0x40 | 0x6A | 0x6D) => {
                Some(Token::OneByte(x))
            }

            Token::OneByte(0x6B) => Some(Token::OneByte(0x6C)),
            Token::OneByte(0x6C) => Some(Token::OneByte(0x6B)),
            Token::OneByte(0x6D) => Some(Token::OneByte(0x6E)),
            Token::OneByte(0x6E) => Some(Token::OneByte(0x6D)),

            _ => None,
        }
    }

    pub fn associative(&self) -> bool {
        matches!(self.kind, Token::OneByte(0x70 | 0x82 | 0x3C | 0x3D | 0x40))
    }

    pub fn recognize_precedence(token: Token) -> Option<u8> {
        match token {
            Token::OneByte(0x3C | 0x3D) => Some(10), // or xor
            Token::OneByte(0x40) => Some(20),        // and
            Token::OneByte(0x6A..=0x6F) => Some(30), // = < > != <= >=
            Token::OneByte(0x70 | 0x71) => Some(40), // + -
            Token::OneByte(0x82 | 0x83) => Some(50), // * /
            Token::OneByte(0x94 | 0x95) => Some(60), // nPr nCr
            Token::OneByte(0xF0 | 0xF1) => Some(70), // ^ xroot
            _ => None,
        }
    }

    pub fn precedence(&self) -> u8 {
        Self::recognize_precedence(self.kind).unwrap()
    }
}

impl Reconstruct for BinOp {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        let mut implicit_mul_viable = true;
        let mut result = match &*self.left {
            Expression::Operator(Operator::Binary(left_binop))
                if left_binop.precedence() < self.precedence() =>
            {
                once(Token::OneByte(0x10))
                    .chain(left_binop.reconstruct(config))
                    .chain(once(Token::OneByte(0x11)))
                    .collect()
            }

            Expression::Operand(operand) => {
                if self.kind == Token::OneByte(0x82)
                    && (ListIndexable::try_from(operand).is_ok()
                        || MatrixIndexable::try_from(operand).is_ok())
                    || (matches!(operand, Operand::NumericLiteral(_))
                        && matches!(
                            &*self.right,
                            Expression::Operand(Operand::NumericLiteral(_))
                        ))
                {
                    implicit_mul_viable = false;
                }

                operand.reconstruct(config)
            }

            expr => expr.reconstruct(config),
        };

        if self.kind != Token::OneByte(0x82) || !implicit_mul_viable {
            result.push(self.kind)
        }

        match &*self.right {
            Expression::Operator(Operator::Binary(right_binop))
                if right_binop.precedence() <= self.precedence()
                    && !(self.kind == right_binop.kind && self.associative()) =>
            {
                result.push(Token::OneByte(0x10));
                result.extend(right_binop.reconstruct(config));
                result.push(Token::OneByte(0x11));
            }

            expr => result.extend(expr.reconstruct(config)),
        }

        result
    }
}

mod tests {
    use super::*;

    #[test]
    fn no_precedence_if_not_binop() {
        assert!(BinOp::recognize_precedence(Token::OneByte(0x10)).is_none())
    }

    #[test]
    fn greater_precedence() {
        // * has greater precedence than +
        assert!(
            BinOp::recognize_precedence(Token::OneByte(0x82)).unwrap()
                > BinOp::recognize_precedence(Token::OneByte(0x70)).unwrap()
        )
    }
}
