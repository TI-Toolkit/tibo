use titokens::{Token, Tokens};

pub mod commands;
pub mod components;
pub mod expression;
mod program;

pub use program::Program;

use crate::{error_reporting::TokenReport, Config};

pub(crate) trait Parse: Sized {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport>;
}

pub(crate) trait Reconstruct {
    fn reconstruct(&self, config: &Config) -> Vec<Token>;
}
