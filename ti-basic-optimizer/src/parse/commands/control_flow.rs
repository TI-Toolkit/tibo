mod for_loop;
mod menu;
mod isds;

use crate::parse::commands::control_flow::menu::Menu;
use crate::parse::expression::Expression;
use crate::parse::Parse;
use titokens::{Token, Tokens};
use crate::parse::commands::control_flow::for_loop::ForLoop;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct LabelName(u16);

impl Parse for LabelName {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        if !token.is_alphanumeric() {
            return None;
        }

        let mut data = (token.byte() as u16) << 8;

        if let Some(tok) = more.peek() {
            if tok.is_alphanumeric() {
                more.next();
                data |= tok.byte() as u16;
            }
        }

        Some(LabelName(data))
    }
}

pub enum ControlFlow {
    If(Expression),
    While(Expression),
    Repeat(Expression),
    For(ForLoop),
    End,
    Return,
    Lbl(LabelName),
    Goto(LabelName),
    Stop,
    Menu(Menu),
}

impl Parse for ControlFlow {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        use ControlFlow as CF;
        use Expression as Expr;
        match token {
            Token::OneByte(0xCE) => Some(CF::If(Expr::parse(more.next().unwrap(), more).unwrap())),
            Token::OneByte(0xD1) => Some(CF::While(Expr::parse(more.next().unwrap(), more).unwrap())),
            Token::OneByte(0xD2) => Some(CF::Repeat(Expr::parse(more.next().unwrap(), more).unwrap())),
            Token::OneByte(0xD3) => Some(CF::For(ForLoop::parse(more.next().unwrap(), more).unwrap())),
            Token::OneByte(0xD4) => Some(CF::End),
            Token::OneByte(0xD5) => Some(CF::Return),
            Token::OneByte(0xD6) => Some(CF::Lbl(LabelName::parse(more.next().unwrap(), more).unwrap())),
            Token::OneByte(0xD7) => Some(CF::Goto(LabelName::parse(more.next().unwrap(), more).unwrap())),
            Token::OneByte(0xD9) => Some(CF::Stop),
            Token::OneByte(0xE6) => Some(CF::Menu(Menu::parse(more.next().unwrap(), more).unwrap())),
            _ => None,
        }
    }
}
