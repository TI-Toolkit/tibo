mod for_loop;
mod isds;
mod menu;

use crate::error_reporting::{expect_some, next_or_err, LineReport};
use crate::parse::{
    commands::control_flow::{for_loop::ForLoop, isds::IsDs, menu::Menu},
    expression::Expression,
    Parse,
};
use titokens::{Token, Tokens};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct LabelName(u16);

impl Parse for LabelName {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        if !token.is_alphanumeric() {
            return Ok(None);
        }

        let mut data = (token.byte() as u16) << 8;

        if let Some(tok) = more.peek() {
            if tok.is_alphanumeric() {
                more.next();
                data |= tok.byte() as u16;
            }
        }

        Ok(Some(LabelName(data)))
    }
}

#[derive(Clone, Debug)]
pub enum ControlFlow {
    If(Expression),
    Then,
    Else,
    While(Expression),
    Repeat(Expression),
    For(ForLoop),
    End,
    Return,
    Lbl(LabelName),
    Goto(LabelName),
    Stop,
    IsGt(IsDs),
    DsLt(IsDs),
    Menu(Menu),
}

impl Parse for ControlFlow {
    #[rustfmt::skip]
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        use ControlFlow as CF;
        use Expression as Expr;

        match token {
            Token::OneByte(0xCE) => Ok(Some(CF::If(expect_some!(Expr::parse(next_or_err!(more)?, more)?, more, "a condition")?))),
            Token::OneByte(0xCF) => Ok(Some(CF::Then)),
            Token::OneByte(0xD0) => Ok(Some(CF::Else)),
            Token::OneByte(0xD1) => Ok(Some(CF::While(expect_some!(Expr::parse(next_or_err!(more)?, more)?, more, "a loop condition")?))),
            Token::OneByte(0xD2) => Ok(Some(CF::Repeat(expect_some!(Expr::parse(next_or_err!(more)?, more)?, more, "a loop condition")?))),
            Token::OneByte(0xD3) => Ok(Some(CF::For(expect_some!(ForLoop::parse(token, more)?, more, "a for statement")?))),
            Token::OneByte(0xD4) => Ok(Some(CF::End)),
            Token::OneByte(0xD5) => Ok(Some(CF::Return)),
            Token::OneByte(0xD6) => Ok(Some(CF::Lbl(expect_some!(LabelName::parse(next_or_err!(more)?, more)?, more, "a label", "All Lbls must be followed by one or two numbers or letters.")?, ))),
            Token::OneByte(0xD7) => Ok(Some(CF::Goto(expect_some!(LabelName::parse(next_or_err!(more)?, more)?, more, "a label", "All Gotos must be followed by one or two numbers or letters.")?,))),
            Token::OneByte(0xD9) => Ok(Some(CF::Stop)),
            Token::OneByte(0xDA) => Ok(Some(CF::IsGt(expect_some!(IsDs::parse(token, more)?, more, "Is<( statement")?))),
            Token::OneByte(0xDB) => Ok(Some(CF::DsLt(expect_some!(IsDs::parse(token, more)?, more, "Ds>( statement")?))),
            Token::OneByte(0xE6) => Ok(Some(CF::Menu(expect_some!(Menu::parse(token, more)?, more, "Menu(")?))),
            _ => Ok(None),
        }
    }
}
