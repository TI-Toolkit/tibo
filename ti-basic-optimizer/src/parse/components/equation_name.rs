use crate::error_reporting::LineReport;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

#[derive(Copy, Clone, Debug)]
pub struct EquationName(Token);

impl Parse for EquationName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        Ok(match token {
            Token::TwoByte(0x5E, 0x10..=0x2B | 0x40..=0x45 | 0x80..=0x82) => {
                Some(EquationName(token))
            }
            _ => None,
        })
    }
}

impl Reconstruct for EquationName {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        vec![self.0]
    }
}
