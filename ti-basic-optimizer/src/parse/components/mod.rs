use crate::error_reporting::{next_or_err, LineReport};
pub use crate::parse::components::{
    binary_operator::BinOp,
    data_access::{ListAccess, ListIndexable, MatrixAccess, MatrixIndexable},
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
mod data_access;
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
    ListAccess(ListAccess),
    MatrixAccess(MatrixAccess),
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
                        Ok(
                            MatrixAccess::parse(name.into(), more.next().unwrap(), more)?
                                .map(Self::MatrixAccess),
                        )
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
                        Ok(ListAccess::parse(name.into(), more.next().unwrap(), more)?
                            .map(Self::ListAccess))
                    } else {
                        Ok(Some(Self::ListName(name)))
                    }
                } else {
                    Ok(None)
                }
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
    ListAccess(ListAccess),
    MatrixAccess(MatrixAccess),
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
            Token::TwoByte(0x5C, _) => {
                if let Some(name) = MatrixName::parse(token, more)? {
                    if more.peek() == Some(Token::OneByte(0x10)) {
                        Ok(
                            MatrixAccess::parse(name.into(), more.next().unwrap(), more)?
                                .map(Self::MatrixAccess),
                        )
                    } else {
                        Ok(Some(Self::Matrix(name)))
                    }
                } else {
                    Ok(None)
                }
            }
            Token::TwoByte(0x5D, _) | Token::OneByte(0xEB) => {
                if let Some(name) = ListName::parse(token, more)? {
                    if more.peek() == Some(Token::OneByte(0x10)) {
                        Ok(ListAccess::parse(name.into(), more.next().unwrap(), more)?
                            .map(Self::ListAccess))
                    } else {
                        Ok(Some(Self::List(name)))
                    }
                } else {
                    Ok(None)
                }
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
    ListAccess(ListAccess),
    MatrixAccess(MatrixAccess),
    ListResizing(ListName),
    MatrixResizing(MatrixName),
    String(StringName),
    // ListAccess, ArrayAccess
    Equation(EquationName),
    WindowVar(WindowVarName),
    RandSeed,
}

impl Parse for StoreTarget {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        match token {
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => {
                Ok(NumericVarName::parse(token, more)?.map(Self::NumericVar))
            }
            Token::TwoByte(0xAA, _) => Ok(StringName::parse(token, more)?.map(Self::String)),
            Token::TwoByte(0x5C, _) => {
                if let Some(name) = MatrixName::parse(token, more)? {
                    if more.peek() == Some(Token::OneByte(0x10)) {
                        Ok(
                            MatrixAccess::parse(name.into(), more.next().unwrap(), more)?
                                .map(Self::MatrixAccess),
                        )
                    } else {
                        Ok(Some(Self::Matrix(name)))
                    }
                } else {
                    Ok(None)
                }
            }
            Token::TwoByte(0x5D, _) | Token::OneByte(0xEB) => {
                if let Some(name) = ListName::parse(token, more)? {
                    if more.peek() == Some(Token::OneByte(0x10)) {
                        Ok(ListAccess::parse(name.into(), more.next().unwrap(), more)?
                            .map(Self::ListAccess))
                    } else {
                        Ok(Some(Self::List(name)))
                    }
                } else {
                    Ok(None)
                }
            }
            Token::OneByte(0xB5) => {
                let next = next_or_err!(more)?;

                if let Some(list) = ListName::parse(next, more)? {
                    Ok(Some(Self::ListResizing(list)))
                } else if let Some(matrix) = MatrixName::parse(next, more)? {
                    Ok(Some(Self::MatrixResizing(matrix)))
                } else {
                    Err(LineReport::new(
                        more.current_position(),
                        "Expected a list or matrix name.",
                        Some("Storing to a dim( of a list or matrix resizes that list or matrix."),
                    ))
                }
            }
            Token::TwoByte(0x5E, _) => Ok(EquationName::parse(token, more)?.map(Self::Equation)),
            Token::TwoByte(0x63, 0x00..=0x2A | 0x32..=0x38) => {
                Ok(WindowVarName::parse(token, more)?.map(Self::WindowVar))
            }
            Token::OneByte(0xAB) => Ok(Some(Self::RandSeed)),
            _ => Ok(None),
        }
    }
}
