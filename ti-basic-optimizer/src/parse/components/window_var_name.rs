use crate::error_reporting::LineReport;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

#[derive(Copy, Clone, Debug)]
pub struct WindowVarName(Token);

impl Parse for WindowVarName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        Ok(match token {
            Token::TwoByte(0x63, 0x00..=0x2A | 0x32..=0x38) => Some(WindowVarName(token)),

            _ => None,
        })
    }
}

impl Reconstruct for WindowVarName {
    fn reconstruct(&self, _version: Version) -> Vec<Token> {
        vec![self.0]
    }
}
