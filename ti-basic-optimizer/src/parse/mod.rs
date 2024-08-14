use titokens::{Token, Tokens};

pub mod components;
pub mod expression;

pub(crate) trait Parse: Sized {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self>;
}

pub enum Command {
    ControlFlow,
    ModeSetting,
    DelVar,
}