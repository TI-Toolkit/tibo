use titokens::{Token, Tokens};

pub use crate::parse::components::{
    binary_operator::BinOp,
    function_call::FunctionCall,
    list_name::ListName,
    list::TIList,
    matrix_name::MatrixName,
    numeric_var_name::NumericVarName,
    string::TIString,
    string_name::StringName,
    unary_operator::UnOp,
    pic_image_name::{ImageName, PicName}
};

use crate::parse::expression::Expression;
use crate::parse::Parse;

mod matrix_name;
mod numeric_var_name;
mod string_name;
mod list_name;
mod numeric_literal;
mod string;
mod function_call;
mod binary_operator;
mod list;
mod unary_operator;
mod pic_image_name;

#[derive(Clone, Debug)]
pub enum Operator {
    Binary(BinOp),
    Unary(UnOp),
    /// The only ternary operator in TI-BASIC.
    MatrixAccess, // TODO
    ListAccess, // TODO

    FunctionCall(FunctionCall),

    /// internal
    #[doc(hidden)]
    LeftParen,
    /// internal
    #[doc(hidden)]
    RightParen,
}

pub(crate) trait OperatorKind {
    fn recognize(token: Token) -> bool;
}

#[derive(Clone, Debug)]
pub enum Operand {
    NumericVarName(NumericVarName),
    ListName(ListName),
    MatrixName(MatrixName),
    StringName(StringName),
    Ans,
    GetKey,
    StartTmr,
    NumericLiteral(tifloats::Float),
    StringLiteral(TIString),
    ListLiteral(TIList),
    /// for expr and seq and such
    Expression(Box<Expression>),
}

impl Parse for Operand {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        match token {
            Token::OneByte(0x30..=0x39 | 0x3A | 0x3B) => {
                tifloats::Float::parse(token, more).map(Self::NumericLiteral)
            }
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => {
                NumericVarName::parse(token, more).map(Self::NumericVarName)
            }
            Token::OneByte(0x72) => Some(Self::Ans),
            Token::OneByte(0xAD) => Some(Self::GetKey),
            Token::TwoByte(0xEF, 0x0B) => Some(Self::StartTmr),
            Token::OneByte(0x2A) => TIString::parse(token, more).map(Self::StringLiteral),
            Token::OneByte(0x08) => TIList::parse(token, more).map(Self::ListLiteral),

            Token::TwoByte(0xAA, _) => StringName::parse(token, more).map(Self::StringName),
            Token::TwoByte(0x5C, _) => MatrixName::parse(token, more).map(Self::MatrixName),
            Token::TwoByte(0x5D, _) | Token::OneByte(0xEB) => {
                ListName::parse(token, more).map(Self::ListName)
            }

            _ => numeric_literal::parse_constant(token, more),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DelVarTarget {
    NumericVar(NumericVarName),
    List(ListName),
    Matrix(MatrixName),
    String(StringName),
    Pic(PicName),
    Image(ImageName),
    // + yvars, GDBs
}

impl Parse for DelVarTarget {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        match token {
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => {
                NumericVarName::parse(token, more).map(Self::NumericVar)
            }
            Token::TwoByte(0xAA, _) => StringName::parse(token, more).map(Self::String),
            Token::TwoByte(0x5C, _) => MatrixName::parse(token, more).map(Self::Matrix),
            Token::TwoByte(0x5D, _) | Token::OneByte(0xEB) => {
                ListName::parse(token, more).map(Self::List)
            },
            Token::TwoByte(0x60, _) => PicName::parse(token, more).map(Self::Pic),
            Token::TwoByte(0xEF, _) => ImageName::parse(token, more).map(Self::Image),
            _ => None
        }
    }
}