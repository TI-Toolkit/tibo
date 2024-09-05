use titokens::{Token, Tokens};

pub mod commands;
pub mod components;
pub mod expression;
mod program;

pub use program::Program;

use crate::{error_reporting::LineReport, Config};

pub(crate) trait Parse: Sized {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport>;
}

pub(crate) trait Reconstruct {
    fn reconstruct(&self, config: &Config) -> Vec<Token>;
}
