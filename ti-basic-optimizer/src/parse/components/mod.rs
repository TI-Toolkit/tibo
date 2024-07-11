use titokens::{Token, Tokens};

use crate::parse::components::list_name::ListName;
use crate::parse::components::matrix_name::MatrixName;
use crate::parse::components::numeric_var_name::NumericVarName;
use crate::parse::components::string::TIString;
use crate::parse::components::string_name::StringName;
use crate::parse::Parse;

mod matrix_name;
mod numeric_var_name;
mod string_name;
mod list_name;
mod numeric_literal;
mod string;

pub enum Component {
    NumericVarName(NumericVarName),
    ListName(ListName),
    MatrixName(MatrixName),
    StringName(StringName),
    Ans,
    NumericLiteral(tifloats::Float),
    StringLiteral(TIString),
    Expression(Vec<Component>),
}

impl Parse for Component {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        match token {
            Token::OneByte(0x30..=0x39 | 0x3A | 0x3B) => {
                tifloats::Float::parse(token, more).map(Self::NumericLiteral)
            }
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => {
                NumericVarName::parse(token, more).map(Self::NumericVarName)
            }
            Token::OneByte(0x72) => Some(Self::Ans),
            Token::OneByte(0x2A) => TIString::parse(token, more).map(Self::StringLiteral),

            Token::TwoByte(0xAA, _) => StringName::parse(token, more).map(Self::StringName),
            Token::TwoByte(0x5C, _) => MatrixName::parse(token, more).map(Self::MatrixName),
            Token::TwoByte(0x5D, _) | Token::OneByte(0xEB) => {
                ListName::parse(token, more).map(Self::ListName)
            }

            _ => todo!() //parse_constant(token, more),
        }
    }
}