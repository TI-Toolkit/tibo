use titokens::{Token, Tokens};

pub mod components;
pub mod expression;
pub mod commands;
mod program;

pub(crate) trait Parse: Sized {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self>;
}

pub(crate) trait Reconstruct {
    fn reconstruct(&self) -> Vec<Token>;
}