use crate::error_reporting::{expect_some, expect_tok, next_or_err, LineReport};
use crate::parse::{
    components::{ListName, MatrixName, Operand},
    expression::Expression,
    Parse,
};
use titokens::{Token, Tokens};

#[derive(Debug, Clone)]
pub enum ListIndexable {
    List(ListName),
    GetDate,
    Ans,
}

impl TryFrom<Operand> for ListIndexable {
    type Error = ();

    fn try_from(value: Operand) -> Result<Self, Self::Error> {
        match value {
            Operand::ListName(x) => Ok(Self::List(x)),
            Operand::Ans => Ok(Self::Ans),
            Operand::GetDate => Ok(Self::GetDate),
            _ => Err(()),
        }
    }
}

impl From<ListName> for ListIndexable {
    fn from(value: ListName) -> Self {
        Self::List(value)
    }
}

#[derive(Debug, Clone)]
pub enum MatrixIndexable {
    Matrix(MatrixName),
    Ans,
}

impl From<MatrixName> for MatrixIndexable {
    fn from(value: MatrixName) -> Self {
        Self::Matrix(value)
    }
}

impl TryFrom<Operand> for MatrixIndexable {
    type Error = ();

    fn try_from(value: Operand) -> Result<Self, Self::Error> {
        match value {
            Operand::MatrixName(x) => Ok(Self::Matrix(x)),
            Operand::Ans => Ok(MatrixIndexable::Ans),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatrixAccess {
    subject: MatrixIndexable,
    row: Box<Expression>,
    col: Box<Expression>,
}

impl MatrixAccess {
    pub fn parse(
        subject: MatrixIndexable,
        token: Token,
        more: &mut Tokens,
    ) -> Result<Option<Self>, LineReport> {
        if token != Token::OneByte(0x10) {
            return Ok(None);
        }
        let row = expect_some!(
            Expression::parse(next_or_err!(more)?, more)?,
            more,
            "an expression",
            "This is a matrix access. Matrix accesses require both a row and column."
        )?;
        expect_tok!(
            more,
            Token::OneByte(0x2B),
            "Expected to find a comma.",
            "This is a matrix access. Matrix accesses require both a row and column."
        )?;
        let col = expect_some!(
            Expression::parse(next_or_err!(more)?, more)?,
            more,
            "an expression",
            "This is a matrix access. Matrix accesses require both a row and column."
        )?;

        if let Some(Token::OneByte(0x11)) = more.peek() {
            more.next();
        }

        Ok(Some(MatrixAccess {
            subject,
            row: Box::new(row),
            col: Box::new(col),
        }))
    }
}

#[derive(Debug, Clone)]
pub struct ListAccess {
    subject: ListIndexable,
    index: Box<Expression>,
}

impl ListAccess {
    pub fn parse(
        subject: ListIndexable,
        token: Token,
        more: &mut Tokens,
    ) -> Result<Option<Self>, LineReport> {
        if token != Token::OneByte(0x10) {
            return Ok(None);
        }

        let index = expect_some!(
            Expression::parse(next_or_err!(more)?, more)?,
            more,
            "an expression",
            "This is a list access and list accesses require an index."
        )?;

        if more.peek() == Some(Token::OneByte(0x11)) {
            more.next();
        }

        Ok(Some(ListAccess {
            subject,
            index: Box::new(index),
        }))
    }
}
