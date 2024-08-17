use crate::error_reporting::LineReport;
pub use crate::parse::components::{
    binary_operator::BinOp,
    equation_name::EquationName,
    function_call::FunctionCall,
    list::TIList,
    list_name::ListName,
    matrix_name::MatrixName,
    numeric_var_name::NumericVarName,
    pic_image_name::{ImageName, PicName},
    string::TIString,
    string_name::StringName,
    unary_operator::UnOp,
    window_var_name::WindowVarName,
};
use crate::parse::expression::Expression;
use crate::parse::Parse;
use titokens::{Token, Tokens};

mod binary_operator;
mod equation_name;
mod function_call;
mod list;
mod list_name;
mod matrix_name;
mod numeric_literal;
mod numeric_var_name;
mod pic_image_name;
mod string;
mod string_name;
mod unary_operator;
mod window_var_name;

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
    GetDate,
    StartTmr,
    NumericLiteral(tifloats::Float),
    StringLiteral(TIString),
    ListLiteral(TIList),
    TblInput,
    WindowVarName(WindowVarName),
    /// for expr and seq and such
    Expression(Box<Expression>),
}

impl Parse for Operand {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        match token {
            Token::OneByte(0x30..=0x39 | 0x3A | 0x3B) => {
                Ok(tifloats::Float::parse(token, more)?.map(Self::NumericLiteral))
            }
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => {
                Ok(NumericVarName::parse(token, more)?.map(Self::NumericVarName))
            }
            Token::OneByte(0x72) => Ok(Some(Self::Ans)),
            Token::OneByte(0xAD) => Ok(Some(Self::GetKey)),
            Token::TwoByte(0xEF, 0x09) => Ok(Some(Self::GetDate)),
            Token::TwoByte(0xEF, 0x0B) => Ok(Some(Self::StartTmr)),
            Token::OneByte(0x2A) => Ok(TIString::parse(token, more)?.map(Self::StringLiteral)),
            Token::OneByte(0x08) => Ok(TIList::parse(token, more)?.map(Self::ListLiteral)),

            Token::TwoByte(0xAA, _) => Ok(StringName::parse(token, more)?.map(Self::StringName)),
            Token::TwoByte(0x5C, _) => Ok(MatrixName::parse(token, more)?.map(Self::MatrixName)),
            Token::TwoByte(0x5D, _) | Token::OneByte(0xEB) => {
                Ok(ListName::parse(token, more)?.map(Self::ListName))
            }
            Token::TwoByte(0x63, 0x2A) => Ok(Some(Self::TblInput)),
            Token::TwoByte(0x63, 0x00..=0x2A | 0x32..=0x38) => {
                Ok(WindowVarName::parse(token, more)?.map(Self::WindowVarName))
            }
            _ => Ok(numeric_literal::parse_constant(token, more)),
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
    // GDBs
    Equation(EquationName),
}

impl Parse for DelVarTarget {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        match token {
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => {
                Ok(NumericVarName::parse(token, more)?.map(Self::NumericVar))
            }
            Token::TwoByte(0xAA, _) => Ok(StringName::parse(token, more)?.map(Self::String)),
            Token::TwoByte(0x5C, _) => Ok(MatrixName::parse(token, more)?.map(Self::Matrix)),
            Token::TwoByte(0x5D, _) | Token::OneByte(0xEB) => {
                Ok(ListName::parse(token, more)?.map(Self::List))
            }
            Token::TwoByte(0x60, _) => Ok(PicName::parse(token, more)?.map(Self::Pic)),
            Token::TwoByte(0xEF, _) => Ok(ImageName::parse(token, more)?.map(Self::Image)),
            Token::TwoByte(0x5E, _) => Ok(EquationName::parse(token, more)?.map(Self::Equation)),
            _ => Ok(None),
        }
    }
}

#[derive(Clone, Debug)]
pub enum StoreTarget {
    NumericVar(NumericVarName),
    List(ListName),
    Matrix(MatrixName),
    String(StringName),
    // ListAccess, ArrayAccess
    Equation(EquationName),
    WindowVar(WindowVarName),
}

impl Parse for StoreTarget {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        match token {
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => {
                Ok(NumericVarName::parse(token, more)?.map(Self::NumericVar))
            }
            Token::TwoByte(0xAA, _) => Ok(StringName::parse(token, more)?.map(Self::String)),
            Token::TwoByte(0x5C, _) => Ok(MatrixName::parse(token, more)?.map(Self::Matrix)),
            Token::TwoByte(0x5D, _) | Token::OneByte(0xEB) => {
                Ok(ListName::parse(token, more)?.map(Self::List))
            }
            Token::TwoByte(0x5E, _) => Ok(EquationName::parse(token, more)?.map(Self::Equation)),
            Token::TwoByte(0x63, 0x00..=0x2A | 0x32..=0x38) => {
                Ok(WindowVarName::parse(token, more)?.map(Self::WindowVar))
            }
            _ => Ok(None),
        }
    }
}
