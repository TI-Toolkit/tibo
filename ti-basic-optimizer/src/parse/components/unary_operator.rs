use titokens::Token;
use crate::parse::components::OperatorKind;
use crate::parse::expression::Expression;

#[derive(Clone, Debug)]
pub struct UnOp {
    pub kind: Token,
    pub child: Box<Expression>,
}

impl OperatorKind for UnOp {
    fn recognize(token: Token) -> bool {
        matches!(token,
            Token::OneByte(0xB0) | // Negate
            Token::OneByte(0x0A..=0x0F) | // Radian, Reciprocal, Squared, Transpose, Cubed
            Token::OneByte(0x2D) | // Factorial
            Token::TwoByte(0xBB, 0xDA) // Percent (undocumented)
        )
    }
}