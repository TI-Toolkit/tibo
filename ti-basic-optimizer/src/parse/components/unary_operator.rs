use crate::parse::components::{BinOp, Operator, OperatorKind};
use crate::parse::expression::Expression;
use crate::parse::Reconstruct;
use titokens::{Token, Version};

#[derive(Clone, Debug)]
pub struct UnOp {
    pub kind: Token,
    pub child: Box<Expression>,
}

impl OperatorKind for UnOp {
    fn recognize(token: Token) -> bool {
        matches!(
            token,
            Token::OneByte(0xB0) | // Negate
            Token::OneByte(0x0A..=0x0F) | // Radian, Reciprocal, Squared, Transpose, Cubed
            Token::OneByte(0x2D) | // Factorial
            Token::TwoByte(0xBB, 0xDA) // Percent (undocumented)
        )
    }
}

impl Reconstruct for UnOp {
    /// Parenthesis around a UnOp `child` are required in the following cases:
    ///
    fn reconstruct(&self, version: &Version) -> Vec<Token> {
        let mut result = vec![];

        // ~
        if self.kind == Token::OneByte(0xB0) {
            result.push(self.kind);
            match *self.child {
                Expression::Operator(Operator::Binary(BinOp { kind, .. })) if !matches!(kind, Token::OneByte(0x82 | 0x83)) /* mul, div */ => {
                    result.push(Token::OneByte(0x10)); // (
                    result.extend(self.child.reconstruct(version));
                    result.push(Token::OneByte(0x10)); // )
                },
                _ => result.extend(self.child.reconstruct(version)),
            }
        } else {
            match *self.child {
                Expression::Operator(
                    Operator::Binary(BinOp { .. })
                    | Operator::Unary(UnOp {
                        kind: Token::OneByte(0xB0),
                        ..
                    }),
                ) => {
                    result.push(Token::OneByte(0x10)); // (
                    result.extend(self.child.reconstruct(version));
                    result.push(Token::OneByte(0x11)); // )
                }
                _ => result.extend(self.child.reconstruct(version)),
            }

            result.push(self.kind);
        };

        result
    }
}
