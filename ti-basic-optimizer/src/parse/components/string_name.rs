use crate::error_reporting::LineReport;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

#[derive(Copy, Clone, Debug)]
pub struct StringName(Token);

impl Parse for StringName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        Ok(match token {
            Token::TwoByte(0xAA, 0x00..=0x0A) => Some(StringName(token)),
            _ => None,
        })
    }
}

impl Reconstruct for StringName {
    fn reconstruct(&self, _version: &Version) -> Vec<Token> {
        vec![self.0]
    }
}
