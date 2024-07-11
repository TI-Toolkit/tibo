use titokens::{Token, Tokens};

use crate::parse::Parse;

#[derive(Copy, Clone, Debug)]
pub enum NumericVarName {
    // $41
    A,
    // $42
    B,
    // $43
    C,
    // $44
    D,
    // $45
    E,
    // $46
    F,
    // $47
    G,
    // $48
    H,
    // $49
    I,
    // $4A
    J,
    // $4B
    K,
    // $4C
    L,
    // $4D
    M,
    // $4E
    N,
    // $4F
    O,
    // $50
    P,
    // $51
    Q,
    // $52
    R,
    // $53
    S,
    // $54
    T,
    // $55
    U,
    // $56
    V,
    // $57
    W,
    // $58
    X,
    // $59
    Y,
    // $5A
    Z,
    // $5B
    Theta,
    // $6221 - often functions the same as the other numeric vars
    RecursiveN,
}

impl Parse for NumericVarName {
    fn parse(token: Token, _more: &mut Tokens) -> Option<Self> {
        match token {
            Token::OneByte(0x41) => Some(Self::A),
            Token::OneByte(0x42) => Some(Self::B),
            Token::OneByte(0x43) => Some(Self::C),
            Token::OneByte(0x44) => Some(Self::D),
            Token::OneByte(0x45) => Some(Self::E),
            Token::OneByte(0x46) => Some(Self::F),
            Token::OneByte(0x47) => Some(Self::G),
            Token::OneByte(0x48) => Some(Self::H),
            Token::OneByte(0x49) => Some(Self::I),
            Token::OneByte(0x4A) => Some(Self::J),
            Token::OneByte(0x4B) => Some(Self::K),
            Token::OneByte(0x4C) => Some(Self::L),
            Token::OneByte(0x4D) => Some(Self::M),
            Token::OneByte(0x4E) => Some(Self::N),
            Token::OneByte(0x4F) => Some(Self::O),
            Token::OneByte(0x50) => Some(Self::P),
            Token::OneByte(0x51) => Some(Self::Q),
            Token::OneByte(0x52) => Some(Self::R),
            Token::OneByte(0x53) => Some(Self::S),
            Token::OneByte(0x54) => Some(Self::T),
            Token::OneByte(0x55) => Some(Self::U),
            Token::OneByte(0x56) => Some(Self::V),
            Token::OneByte(0x57) => Some(Self::W),
            Token::OneByte(0x58) => Some(Self::X),
            Token::OneByte(0x59) => Some(Self::Y),
            Token::OneByte(0x5A) => Some(Self::Z),
            Token::OneByte(0x5B) => Some(Self::Theta),
            Token::TwoByte(0x62, 0x21) => Some(Self::RecursiveN),

            _ => None,
        }
    }
}
