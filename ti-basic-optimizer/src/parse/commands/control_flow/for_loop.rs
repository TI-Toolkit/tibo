use crate::error_reporting::{expect_some, expect_tok, next_or_err, LineReport};
use crate::parse::expression::Expression;
use crate::parse::Parse;
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub struct ForLoop {
    pub iterator: Expression,
    pub start: Expression,
    pub end: Expression,
    pub step: Option<Expression>,
}

impl Parse for ForLoop {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        if token != Token::OneByte(0xD3) {
            return Ok(None);
        }

        let iterator = expect_some!(
            Expression::parse(next_or_err!(more)?, more)?,
            more,
            "an expression"
        )?;
        expect_tok!(
            more,
            Token::OneByte(0x2B),
            "Expected a comma.",
            "For loops have at least 3 arguments."
        )?;
        let start = expect_some!(
            Expression::parse(next_or_err!(more)?, more)?,
            more,
            "an expression"
        )?;
        expect_tok!(
            more,
            Token::OneByte(0x2B),
            "Expected a comma.",
            "For loops have at least 3 arguments."
        )?;
        let end = expect_some!(
            Expression::parse(next_or_err!(more)?, more)?,
            more,
            "an expression"
        )?;

        let mut step = None;

        if more.peek() == Some(Token::OneByte(0x2B)) {
            more.next();
            step = Some(expect_some!(
                Expression::parse(next_or_err!(more)?, more)?,
                more,
                "an expression"
            )?);
        }

        if more.peek() == Some(Token::OneByte(0x11)) {
            more.next();
        }

        Ok(Some(ForLoop {
            iterator,
            start,
            end,
            step,
        }))
    }
}
