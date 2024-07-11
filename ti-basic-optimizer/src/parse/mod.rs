use titokens::{Token, Tokens};

use crate::parse::components::Component;

mod components;
mod expression;

pub(crate) trait Parse: Sized {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self>;
}

pub enum Command {
    ControlFlow,
    ModeSetting,
    DelVar,
}