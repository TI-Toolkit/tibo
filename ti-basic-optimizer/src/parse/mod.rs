use titokens::{Token, Tokens};

pub mod commands;
pub mod components;
pub mod expression;
mod program;

pub(crate) trait Parse: Sized {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self>;
}

pub(crate) trait Reconstruct {
    fn reconstruct(&self) -> Vec<Token>;
}
