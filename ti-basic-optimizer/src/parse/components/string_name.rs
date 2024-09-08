use crate::error_reporting::TokenReport;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

#[derive(Copy, Clone, Debug)]
pub struct StringName(Token);

impl Parse for StringName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        Ok(match token {
            Token::TwoByte(0xAA, 0x00..=0x0A) => Some(StringName(token)),
            _ => None,
        })
    }
}

impl Reconstruct for StringName {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        vec![self.0]
    }
}
