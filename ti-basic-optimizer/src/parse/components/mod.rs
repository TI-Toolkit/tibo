use crate::error_reporting::LineReport;
pub use crate::parse::components::{
    binary_operator::BinOp,
    data_access::{ListIndex, ListIndexable, MatrixIndex, MatrixIndexable},
    delvar_target::DelVarTarget,
    equation_name::EquationName,
    function_call::FunctionCall,
    list::TIList,
    list_name::{ListName, DEFAULT_LISTS},
    matrix_name::MatrixName,
    numeric_var_name::NumericVarName,
    pic_image_name::{ImageName, PicName},
    store_target::StoreTarget,
    string::TIString,
    string_name::StringName,
    unary_operator::UnOp,
    window_var_name::WindowVarName,
};
use crate::parse::expression::Expression;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

mod binary_operator;
mod data_access;
mod delvar_target;
mod equation_name;
mod function_call;
mod list;
mod list_name;
mod matrix_name;
mod numeric_literal;
mod numeric_var_name;
mod pic_image_name;
mod store_target;
mod string;
mod string_name;
mod unary_operator;
mod window_var_name;

#[derive(Clone, Debug)]
pub enum Operator {
    Binary(BinOp),
    Unary(UnOp),

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

impl Reconstruct for Operator {
    fn reconstruct(&self, version: &Version) -> Vec<Token> {
        match self {
            Operator::Binary(binop) => binop.reconstruct(version),
            Operator::Unary(unop) => unop.reconstruct(version),
            Operator::FunctionCall(function_call) => function_call.reconstruct(version),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Operand {
    NumericVarName(NumericVarName),
    ListName(ListName),
    MatrixName(MatrixName),
    ListAccess(ListIndex),
    MatrixAccess(MatrixIndex),
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
            Token::TwoByte(0x5C, _) => {
                if let Some(name) = MatrixName::parse(token, more)? {
                    if more.peek() == Some(Token::OneByte(0x10)) {
                        Ok(MatrixIndex::parse(name.into(), more.next().unwrap(), more)?
                            .map(Self::MatrixAccess))
                    } else {
                        Ok(Some(Self::MatrixName(name)))
                    }
                } else {
                    Ok(None)
                }
            }
            Token::TwoByte(0x5D, _) | Token::OneByte(0xEB) => {
                if let Some(name) = ListName::parse(token, more)? {
                    if more.peek() == Some(Token::OneByte(0x10)) {
                        Ok(ListIndex::parse(name.into(), more.next().unwrap(), more)?
                            .map(Self::ListAccess))
                    } else {
                        Ok(Some(Self::ListName(name)))
                    }
                } else {
                    Ok(None)
                }
            }
            Token::TwoByte(0x63, 0x2A) => Ok(Some(Self::TblInput)), // todo: TblIndex(n) list access
            Token::TwoByte(0x63, 0x00..=0x2A | 0x32..=0x38) => {
                Ok(WindowVarName::parse(token, more)?.map(Self::WindowVarName))
            }
            _ => Ok(numeric_literal::parse_constant(token, more)),
        }
    }
}

impl Reconstruct for Operand {
    fn reconstruct(&self, version: &Version) -> Vec<Token> {
        match self {
            Operand::NumericVarName(x) => x.reconstruct(version),
            Operand::ListName(x) => x.reconstruct(version),
            Operand::MatrixName(x) => x.reconstruct(version),
            Operand::ListAccess(x) => x.reconstruct(version),
            Operand::MatrixAccess(x) => x.reconstruct(version),
            Operand::StringName(x) => x.reconstruct(version),
            Operand::Ans => vec![Token::OneByte(0x72)],
            Operand::GetKey => vec![Token::OneByte(0xAD)],
            Operand::GetDate => vec![Token::TwoByte(0xEF, 0x09)],
            Operand::StartTmr => vec![Token::TwoByte(0xEF, 0x0B)],
            Operand::NumericLiteral(x) => x.reconstruct(version),
            Operand::StringLiteral(x) => x.reconstruct(version),
            Operand::ListLiteral(x) => x.reconstruct(version),
            Operand::TblInput => vec![Token::TwoByte(0x63, 0x2A)],
            Operand::WindowVarName(x) => x.reconstruct(version),
            Operand::Expression(x) => x.reconstruct(version),
        }
    }
}
