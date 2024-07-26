use titokens::Token;
use crate::parse::components::OperatorKind;
use crate::parse::expression::Expression;

#[derive(Clone, Debug)]
pub struct BinOp {
    pub kind: Token,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl OperatorKind for BinOp {
    fn recognize(token: Token) -> bool {
        matches!(token,
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
    pub fn recognize_precedence(token: Token) -> u8 {
        match token {
            Token::OneByte(0x3C | 0x3D) => 10, // or xor
            Token::OneByte(0x40) => 20, // and
            Token::OneByte(0x6A..=0x6F) => 30, // = < > != <= >=
            Token::OneByte(0x70 | 0x71) => 40, // + -
            Token::OneByte(0x82 | 0x83) => 50, // * /
            Token::OneByte(0x94 | 0x95) => 60, // nPr nCr
            Token::OneByte(0xF0 | 0xF1) => 70, // ^ xroot
            _ => 0
        }
    }

    pub fn precedence(&self) -> u8 {
        Self::recognize_precedence(self.kind)
    }
}

mod tests {
    use super::*;

    #[test]
    fn zero_precedence_if_not_binop() {
        assert_eq!(BinOp::recognize_precedence(Token::OneByte(0x10)), 0)
    }

    #[test]
    fn greater_precedence() {
        // * has greater precedence than +
        assert!(BinOp::recognize_precedence(Token::OneByte(0x82)) > BinOp::recognize_precedence(Token::OneByte(0x70)))
    }
}