use crate::error_reporting::LineReport;
use crate::parse::{
    components::{
        EquationName, ImageName, ListIndex, ListName, MatrixIndex, MatrixName, NumericVarName,
        PicName, StringName,
    },
    Parse, Reconstruct,
};
use titokens::{Token, Tokens, Version};

#[derive(Clone, Debug)]
pub enum DelVarTarget {
    NumericVar(NumericVarName),
    List(ListName),
    Matrix(MatrixName),
    ListAccess(ListIndex),
    MatrixAccess(MatrixIndex),
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
                            MatrixIndex::parse(name.into(), more.next().unwrap(), more)?
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
                        Ok(ListIndex::parse(name.into(), more.next().unwrap(), more)?
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

impl Reconstruct for DelVarTarget {
    fn reconstruct(&self, version: &Version) -> Vec<Token> {
        match self {
            DelVarTarget::NumericVar(x) => x.reconstruct(version),
            DelVarTarget::List(x) => x.reconstruct(version),
            DelVarTarget::Matrix(x) => x.reconstruct(version),
            DelVarTarget::ListAccess(x) => x.reconstruct(version),
            DelVarTarget::MatrixAccess(x) => x.reconstruct(version),
            DelVarTarget::String(x) => x.reconstruct(version),
            DelVarTarget::Pic(x) => x.reconstruct(version),
            DelVarTarget::Image(x) => x.reconstruct(version),
            DelVarTarget::Equation(x) => x.reconstruct(version),
        }
    }
}
