use crate::error_reporting::TokenReport;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

#[derive(Copy, Clone, Debug)]
pub struct PicName(Token);

impl Parse for PicName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        Ok(match token {
            Token::TwoByte(0x60, 0x00..=0x0A) => Some(PicName(token)),
            _ => None,
        })
    }
}

impl Reconstruct for PicName {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        vec![self.0]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ImageName(Token);

impl Parse for ImageName {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        Ok(match token {
            Token::TwoByte(0xEF, 0x50..=0x59) => Some(ImageName(token)),
            _ => None,
        })
    }
}

impl Reconstruct for ImageName {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        vec![self.0]
    }
}
