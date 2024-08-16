use titokens::{Token, Tokens};

use crate::parse::Parse;

#[derive(Copy, Clone, Debug)]
pub struct StringName(Token);

impl Parse for StringName {
    fn parse(token: Token, _more: &mut Tokens) -> Option<Self> {
        match token {
            Token::TwoByte(0xAA, 0x00..=0x0A) => Some(StringName(token)),
            _ => None,
        }
    }
}
