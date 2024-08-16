use titokens::{Token, Tokens};

use crate::parse::{commands::Command, components::DelVarTarget, Parse};

pub struct DelVarChain {
    pub deletions: Vec<DelVarTarget>,
    pub valence: Option<Command>,
}

impl Parse for DelVarChain {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        (token == Token::TwoByte(0xBB, 0x54)).then(|| {
            let mut chain = DelVarChain {
                deletions: vec![],
                valence: None,
            };

            while let Some(deletion) = DelVarTarget::parse(more.next().unwrap(), more) {
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
                chain.valence = Command::parse(tok, more);
            }

            chain
        })
    }
}
