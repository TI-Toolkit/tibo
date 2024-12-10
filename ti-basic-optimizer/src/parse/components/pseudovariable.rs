use crate::error_reporting::TokenReport;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

/// Pseudo-variables are like `GetKey` and `IsClockOn`- functions that return a value and never
/// accept arguments.
#[derive(Clone, Debug)]
pub struct PseudoVariable {
    pub kind: Token,
}

impl Parse for PseudoVariable {
    fn parse(token: Token, _more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        if matches!(
            token,
            Token::OneByte(0xAD) | Token::TwoByte(0xEF, 0x09..=0x0E)
        ) {
            Ok(Some(PseudoVariable { kind: token }))
        } else {
            Ok(None)
        }
    }
}

impl Reconstruct for PseudoVariable {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        vec![self.kind]
    }
}
