use titokens::{Token, Tokens};

use crate::parse::Parse;

#[derive(Copy, Clone, Debug)]
pub enum StringName {
    // $AA00
    Str1,
    // $AA01
    Str2,
    // $AA02
    Str3,
    // $AA03
    Str4,
    // $AA04
    Str5,
    // $AA05
    Str6,
    // $AA06
    Str7,
    // $AA07
    Str8,
    // $AA08
    Str9,
    // $AA09
    Str0,
}

impl Parse for StringName {
    fn parse(token: Token, _more: &mut Tokens) -> Option<Self> {
        match token {
            Token::TwoByte(0xAA, x) if x < 0x0A => match x {
                0x00 => Some(Self::Str1),
                0x01 => Some(Self::Str2),
                0x02 => Some(Self::Str3),
                0x03 => Some(Self::Str4),
                0x04 => Some(Self::Str5),
                0x05 => Some(Self::Str6),
                0x06 => Some(Self::Str7),
                0x07 => Some(Self::Str8),
                0x08 => Some(Self::Str9),
                0x09 => Some(Self::Str0),
                // # Safety
                // x < 0x0A condition above prevents this from being reachable.
                _ => unsafe { core::hint::unreachable_unchecked() },
            },
            _ => None,
        }
    }
}
