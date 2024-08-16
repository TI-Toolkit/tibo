use titokens::{Token, Tokens};

use crate::parse::{Parse, Reconstruct};

#[derive(Copy, Clone, Debug)]
pub struct MatrixName(Token);

impl Parse for MatrixName {
    fn parse(token: Token, _more: &mut Tokens) -> Option<Self> {
        match token {
            Token::TwoByte(0x5C, 0x00..=0x08) => Some(MatrixName(token)),
            _ => None,
        }
    }
}

impl Reconstruct for MatrixName {
    fn reconstruct(&self) -> Vec<Token> {
        vec![self.0]
    }
}