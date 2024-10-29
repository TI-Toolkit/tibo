use itertools::Itertools;
use titokens::{Token, Tokens};

pub mod statements;
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

#[cfg(feature = "debug-tools")]
pub trait Stringify {
    fn stringify(&self, config: Option<&titokens::Tokenizer>) -> String;
}

#[cfg(feature = "debug-tools")]
impl<T> Stringify for T
where
    T: Reconstruct,
{
    fn stringify(&self, config: Option<&titokens::Tokenizer>) -> String {
        match config {
            Some(tokenizer) => tokenizer
                .stringify(&self.reconstruct(&test_files::test_version!().into()))
                .to_string(),
            None => self
                .reconstruct(&test_files::test_version!().into())
                .iter()
                .map(|token| match token {
                    Token::OneByte(b) => format!("{:02X}", b),
                    Token::TwoByte(a, b) => format!("{:02X}{:02X}", a, b),
                })
                .join("\u{202f}"), // NNBSP
        }
    }
}
