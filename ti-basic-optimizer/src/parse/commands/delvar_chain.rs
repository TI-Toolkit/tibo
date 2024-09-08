use crate::error_reporting::{next_or_err, TokenReport};
use crate::parse::{commands::Command, components::DelVarTarget, Parse, Reconstruct};
use crate::Config;
use std::iter::once;
use titokens::{Token, Tokens};

/// `DelVar` statements do not require a trailing newline, and so a series of `deletions` can be
/// chained back-to-back.
#[derive(Clone, Debug)]
pub struct DelVarChain {
    pub deletions: Vec<DelVarTarget>,
    pub valence: Option<Box<Command>>,
}

impl Parse for DelVarChain {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        if token != Token::TwoByte(0xBB, 0x54) {
            return Ok(None);
        }
        let mut chain = DelVarChain {
            deletions: vec![],
            valence: None,
        };

        while let Some(deletion) =
            DelVarTarget::parse(next_or_err!(more, "Expected a DelVar target")?, more)?
        {
            chain.deletions.push(deletion);

            if let Some(Token::TwoByte(0xBB, 0x54)) = more.peek() {
                more.next();
                continue;
            } else {
                break;
            }
        }

        if !matches!(more.peek(), None | Some(Token::OneByte(0x3E | 0x3F))) {
            chain.valence = Command::parse(next_or_err!(more)?, more)?.map(Box::new);
        }

        Ok(Some(chain))
    }
}

impl Reconstruct for DelVarChain {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        self.deletions
            .iter()
            .flat_map(|target| once(Token::TwoByte(0xBB, 0x54)).chain(target.reconstruct(config)))
            .chain(if let Some(val_expr) = &self.valence {
                val_expr.reconstruct(config)
            } else {
                vec![]
            })
            .collect()
    }
}
