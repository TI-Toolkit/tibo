use crate::error_reporting::{expect_some, expect_tok, next_or_err, LineReport};
use crate::parse::{components::{ListName, MatrixName, Operand}, expression::Expression, Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

#[derive(Debug, Clone)]
pub enum ListIndexable {
    List(ListName),
    TblInput,
    Ans,
}

impl From<ListName> for ListIndexable {
    fn from(value: ListName) -> Self {
        Self::List(value)
    }
}

impl Reconstruct for ListIndexable {
    fn reconstruct(&self, version: Version) -> Vec<Token> {
        match self {
            Self::List(name) => name.reconstruct(version),
            Self::TblInput => vec![Token::TwoByte(0x63, 0x2A)],
            Self::Ans => vec![Token::OneByte(0x72)],
        }
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

impl Reconstruct for MatrixIndexable {
    fn reconstruct(&self, version: Version) -> Vec<Token> {
        match self {
            Self::Matrix(name) => name.reconstruct(version),
            Self::Ans => vec![Token::OneByte(0x72)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatrixAccess {
    pub subject: MatrixIndexable,
    pub row: Box<Expression>,
    pub col: Box<Expression>,
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
    pub subject: ListIndexable,
    pub index: Box<Expression>,
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
