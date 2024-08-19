use crate::error_reporting::LineReport;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

#[derive(Copy, Clone, Debug)]
pub struct NumericVarName(Token);

impl Parse for NumericVarName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        Ok(match token {
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => Some(NumericVarName(token)),

            _ => None,
        })
    }
}

impl Reconstruct for NumericVarName {
    fn reconstruct(&self, _version: Version) -> Vec<Token> {
        vec![self.0]
    }
}
