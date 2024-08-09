use titokens::{Token, Tokens};

use crate::parse::Parse;

#[derive(Copy, Clone, Debug)]
pub enum PicName {
    // $6000
    Pic1,
    // $6001
    Pic2,
    // $6002
    Pic3,
    // $6003
    Pic4,
    // $6004
    Pic5,
    // $6005
    Pic6,
    // $6006
    Pic7,
    // $6007
    Pic8,
    // $6008
    Pic9,
    // $6009
    Pic0,
}

impl Parse for PicName {
    fn parse(token: Token, _more: &mut Tokens) -> Option<Self> {
        match token {
            Token::TwoByte(0x60, x) if x < 0x0A => match x {
                0x00 => Some(Self::Pic1),
                0x01 => Some(Self::Pic2),
                0x02 => Some(Self::Pic3),
                0x03 => Some(Self::Pic4),
                0x04 => Some(Self::Pic5),
                0x05 => Some(Self::Pic6),
                0x06 => Some(Self::Pic7),
                0x07 => Some(Self::Pic8),
                0x08 => Some(Self::Pic9),
                0x09 => Some(Self::Pic0),
                // # Safety
                // x < 0x0A condition above prevents this from being reachable.
                _ => unsafe { core::hint::unreachable_unchecked() },
            },
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ImageName {
    // $EF50
    Image1,
    // $EF51
    Image2,
    // $EF52
    Image3,
    // $EF53
    Image4,
    // $EF54
    Image5,
    // $EF55
    Image6,
    // $EF56
    Image7,
    // $EF57
    Image8,
    // $EF58
    Image9,
    // $EF59
    Image0,
}

impl Parse for ImageName {
    fn parse(token: Token, _more: &mut Tokens) -> Option<Self> {
        match token {
            Token::TwoByte(0xEF, x) if (0x50..=0x59).contains(&x) => match x {
                0x50 => Some(Self::Image1),
                0x51 => Some(Self::Image2),
                0x52 => Some(Self::Image3),
                0x53 => Some(Self::Image4),
                0x54 => Some(Self::Image5),
                0x55 => Some(Self::Image6),
                0x56 => Some(Self::Image7),
                0x57 => Some(Self::Image8),
                0x58 => Some(Self::Image9),
                0x59 => Some(Self::Image0),
                _ => None,
            },
            _ => None,
        }
    }
}
