use crate::error_reporting::LineReport;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens};

#[derive(Copy, Clone, Debug)]
pub struct MatrixName(Token);

impl Parse for MatrixName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        Ok(match token {
            Token::TwoByte(0x5C, 0x00..=0x08) => Some(MatrixName(token)),
            _ => None,
        })
    }
}

impl Reconstruct for MatrixName {
    fn reconstruct(&self) -> Vec<Token> {
        vec![self.0]
    }
}
