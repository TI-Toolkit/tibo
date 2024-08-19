use crate::error_reporting::LineReport;
use crate::parse::components::NumericVarName;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

#[derive(Copy, Clone, Debug)]
pub struct PicName(Token);

impl Parse for PicName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        Ok(match token {
            Token::TwoByte(0x60, 0x00..=0x0A) => Some(PicName(token)),
            _ => None,
        })
    }
}

impl Reconstruct for PicName {
    fn reconstruct(&self, version: Version) -> Vec<Token> {
        vec![self.0]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ImageName(Token);

impl Parse for ImageName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        Ok(match token {
            Token::TwoByte(0xEF, 0x50..=0x59) => Some(ImageName(token)),
            _ => None,
        })
    }
}

impl Reconstruct for ImageName {
    fn reconstruct(&self, version: Version) -> Vec<Token> {
        vec![self.0]
    }
}
