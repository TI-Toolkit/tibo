use titokens::{Token, Tokens};

pub mod commands;
pub mod components;
pub mod expression;
mod program;

use crate::error_reporting::LineReport;
pub use program::Program;

pub(crate) trait Parse: Sized {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport>;
}

pub(crate) trait Reconstruct {
    fn reconstruct(&self) -> Vec<Token>;
}
