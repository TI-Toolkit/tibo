use crate::error_reporting::TokenReport;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

#[derive(Copy, Clone, Debug)]
pub struct MatrixName(Token);

impl Parse for MatrixName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        Ok(match token {
            Token::TwoByte(0x5C, 0x00..=0x08) => Some(MatrixName(token)),
            _ => None,
        })
    }
}

impl Reconstruct for MatrixName {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        vec![self.0]
    }
}
