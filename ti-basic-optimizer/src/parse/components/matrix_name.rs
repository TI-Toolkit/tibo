use titokens::{Token, Tokens};

use crate::parse::Parse;

#[derive(Copy, Clone, Debug)]
pub enum MatrixName {
    // $5C00
    A,
    // $5C01
    B,
    // $5C02
    C,
    // $5C03
    D,
    // $5C04
    E,
    // $5C05
    F,
    // $5C06
    G,
    // $5C07
    H,
    // $5C08
    I,

    Ans,
}

impl Parse for MatrixName {
    fn parse(token: Token, _more: &mut Tokens) -> Option<Self> {
        match token {
            Token::TwoByte(0x5C, x) if x < 0x09 => match x {
                0x00 => Some(MatrixName::A),
                0x01 => Some(MatrixName::B),
                0x02 => Some(MatrixName::C),
                0x03 => Some(MatrixName::D),
                0x04 => Some(MatrixName::E),
                0x05 => Some(MatrixName::F),
                0x06 => Some(MatrixName::G),
                0x07 => Some(MatrixName::H),
                0x08 => Some(MatrixName::I),
                // # Safety
                // x < 0x09 condition above prevents this from being reachable.
                _ => unsafe { core::hint::unreachable_unchecked() },
            },
            _ => None,
        }
    }
}
