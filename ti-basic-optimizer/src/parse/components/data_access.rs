use crate::error_reporting::{expect_some, expect_tok, next_or_err, LineReport};
use crate::parse::{
    components::{ListName, MatrixName, Operand},
    expression::Expression,
    Parse, Reconstruct,
};
use crate::Config;
use titokens::{Token, Tokens};

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

impl TryFrom<&Operand> for ListIndexable {
    type Error = ();

    fn try_from(value: &Operand) -> Result<Self, Self::Error> {
        match value {
            Operand::ListName(x) => Ok(Self::List(*x)),
            Operand::Ans => Ok(Self::Ans),
            _ => Err(()),
        }
    }
}

impl Reconstruct for ListIndexable {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        match self {
            Self::List(name) => name.reconstruct(config),
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
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        match self {
            Self::Matrix(name) => name.reconstruct(config),
            Self::Ans => vec![Token::OneByte(0x72)],
        }
    }
}

impl TryFrom<&Operand> for MatrixIndexable {
    type Error = ();

    fn try_from(value: &Operand) -> Result<Self, Self::Error> {
        match value {
            Operand::MatrixName(x) => Ok(Self::Matrix(*x)),
            Operand::Ans => Ok(Self::Ans),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatrixIndex {
    pub subject: MatrixIndexable,
    pub row: Box<Expression>,
    pub col: Box<Expression>,
}

impl MatrixIndex {
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

        Ok(Some(MatrixIndex {
            subject,
            row: Box::new(row),
            col: Box::new(col),
        }))
    }
}

impl Reconstruct for MatrixIndex {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        let mut data = self.subject.reconstruct(config);
        data.push(Token::OneByte(0x10));
        data.extend(self.row.reconstruct(config));
        data.push(Token::OneByte(0x2B));
        data.extend(self.col.reconstruct(config));
        data.push(Token::OneByte(0x11));

        data
    }
}

#[derive(Debug, Clone)]
pub struct ListIndex {
    pub subject: ListIndexable,
    pub index: Box<Expression>,
}

impl ListIndex {
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

        Ok(Some(ListIndex {
            subject,
            index: Box::new(index),
        }))
    }
}

impl Reconstruct for ListIndex {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        let mut data = self.subject.reconstruct(config);
        data.push(Token::OneByte(0x10));
        data.extend(self.index.reconstruct(config));
        data.push(Token::OneByte(0x11));

        data
    }
}
