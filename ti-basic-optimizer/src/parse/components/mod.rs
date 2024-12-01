use crate::error_reporting::{expect_some, next_or_err, TokenReport};
pub use crate::parse::components::{
    binary_operator::BinOp,
    data_access::{EquationIndex, ListIndex, ListIndexable, MatrixIndex, MatrixIndexable},
    delvar_target::DelVarTarget,
    equation_name::EquationName,
    function_call::FunctionCall,
    list::TIList,
    list_name::{ListName, DEFAULT_LISTS},
    matrix_name::MatrixName,
    numeric_var_name::NumericVarName,
    pic_image_name::{ImageName, PicName},
    rand::Rand,
    store_target::StoreTarget,
    string::TIString,
    string_name::StringName,
    unary_operator::UnOp,
    window_var_name::WindowVarName,
};
use crate::parse::expression::Expression;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

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
mod rand;
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
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        match self {
            Operator::Binary(binop) => binop.reconstruct(config),
            Operator::Unary(unop) => unop.reconstruct(config),
            Operator::FunctionCall(function_call) => function_call.reconstruct(config),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Operand {
    NumericVarName(NumericVarName),
    ListName(ListName),
    MatrixName(MatrixName),
    StringName(StringName),
    EquationName(EquationName),
    ListAccess(ListIndex),
    MatrixAccess(MatrixIndex),
    EquationAccess(EquationIndex),
    Ans,
    I,
    Rand(Rand),
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
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        match token {
            Token::OneByte(0x30..=0x39 | 0x3A | 0x3B) => {
                Ok(tifloats::Float::parse(token, more)?.map(Self::NumericLiteral))
            }
            Token::OneByte(0x41..=0x5B) | Token::TwoByte(0x62, 0x21) => {
                Ok(NumericVarName::parse(token, more)?.map(Self::NumericVarName))
            }
            Token::OneByte(0x72) => {
                if more.peek() == Some(Token::OneByte(0x10)) {
                    // (
                    more.next();
                    // conservatively guess that this is a list or matrix access- we can maybe demote it to a muliplication later
                    let index = expect_some!(
                        Expression::parse(next_or_err!(more)?, more)?,
                        more,
                        "an expression"
                    )?;

                    match more.peek() {
                        Some(Token::OneByte(0x2B)) => {
                            // , -> matrix access
                            todo!()
                        }
                        Some(Token::OneByte(0x11)) => {
                            // )
                            more.next();
                        }

                        _ => {}
                    };

                    Ok(Some(Self::ListAccess(ListIndex {
                        subject: ListIndexable::Ans,
                        index: Box::new(index),
                    })))
                } else {
                    Ok(Some(Self::Ans))
                }
            }
            Token::OneByte(0x2C) => Ok(Some(Self::I)),
            Token::OneByte(0xAB) => Ok(Rand::parse(token, more)?.map(Self::Rand)),
            Token::OneByte(0xAD) => Ok(Some(Self::GetKey)),
            Token::TwoByte(0xEF, 0x09) => Ok(Some(Self::GetDate)),
            Token::TwoByte(0xEF, 0x0B) => Ok(Some(Self::StartTmr)),
            Token::OneByte(0x2A) => Ok(TIString::parse(token, more)?.map(Self::StringLiteral)),
            Token::OneByte(0x08) => Ok(TIList::parse(token, more)?.map(Self::ListLiteral)),
            Token::OneByte(0x06) => Ok(TIMatrix::parse(token, more)?.map(Self::MatrixLiteral)),
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
            Token::TwoByte(0x5E, 0x10..=0x2B | 0x40..=0x45 | 0x80..=0x82) => {
                if let Some(name) = EquationName::parse(token, more)? {
                    if more.peek() == Some(Token::OneByte(0x10)) {
                        Ok(EquationIndex::parse(name, more.next().unwrap(), more)?
                            .map(Self::EquationAccess))
                    } else {
                        Ok(Some(Self::EquationName(name)))
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
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        match self {
            Operand::NumericVarName(x) => x.reconstruct(config),
            Operand::ListName(x) => x.reconstruct(config),
            Operand::MatrixName(x) => x.reconstruct(config),
            Operand::StringName(x) => x.reconstruct(config),
            Operand::EquationName(x) => x.reconstruct(config),
            Operand::ListAccess(x) => x.reconstruct(config),
            Operand::MatrixAccess(x) => x.reconstruct(config),
            Operand::EquationAccess(x) => x.reconstruct(config),
            Operand::Ans => vec![Token::OneByte(0x72)],
            Operand::I => vec![Token::OneByte(0x2C)],
            Operand::Rand(x) => x.reconstruct(config),
            Operand::GetKey => vec![Token::OneByte(0xAD)],
            Operand::GetDate => vec![Token::TwoByte(0xEF, 0x09)],
            Operand::StartTmr => vec![Token::TwoByte(0xEF, 0x0B)],
            Operand::NumericLiteral(x) => x.reconstruct(config),
            Operand::StringLiteral(x) => x.reconstruct(config),
            Operand::ListLiteral(x) => x.reconstruct(config),
            Operand::TblInput => vec![Token::TwoByte(0x63, 0x2A)],
            Operand::WindowVarName(x) => x.reconstruct(config),
            Operand::Expression(x) => x.reconstruct(config),
        }
    }
}

impl From<NumericVarName> for Operand {
    fn from(value: NumericVarName) -> Self {
        Operand::NumericVarName(value)
    }
}

impl From<tifloats::Float> for Operand {
    fn from(value: tifloats::Float) -> Self {
        Operand::NumericLiteral(value)
    }
}
