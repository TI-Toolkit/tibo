use crate::error_reporting::LineReport;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

#[derive(Copy, Clone, Debug)]
pub struct NumericVarName(pub Token);

impl Parse for NumericVarName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        Ok(match token {
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => Some(NumericVarName(token)),

            _ => None,
        })
    }
}

impl Reconstruct for NumericVarName {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        vec![self.0]
    }
}
