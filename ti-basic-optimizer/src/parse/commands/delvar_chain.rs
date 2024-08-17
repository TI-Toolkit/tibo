use crate::error_reporting::{next_or_err, LineReport};
use crate::parse::{commands::Command, components::DelVarTarget, Parse};
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub struct DelVarChain {
    pub deletions: Vec<DelVarTarget>,
    pub valence: Option<Box<Command>>,
}

impl Parse for DelVarChain {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
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

        if let Some(tok) = more.peek() {
            more.next();
            chain.valence = Command::parse(tok, more)?.map(Box::new);
        }

        Ok(Some(chain))
    }
}
