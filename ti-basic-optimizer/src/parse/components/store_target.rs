use crate::error_reporting::{next_or_err, LineReport};
use crate::parse::{
    components::{
        EquationName, ListAccess, ListName, MatrixAccess, MatrixName, NumericVarName, StringName,
        WindowVarName,
    },
    Parse,
};
use titokens::{Token, Tokens};

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
