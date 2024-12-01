use std::iter;

use titokens::{Token, Tokens};

use crate::{
    error_reporting::TokenReport,
    parse::{
        expression::Expression,
        Parse,
    },
};

use super::{expect_some, next_or_err, Reconstruct};

#[derive(Clone, Debug)]
pub struct Rand {
    pub count: Option<Box<Expression>>,
}

impl Rand {
    pub fn is_list(&self) -> bool {
        self.count.is_some()
    }
}

impl Parse for Rand {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        if token != Token::OneByte(0xAB) {
            return Ok(None);
        }

        if more.peek() == Some(Token::OneByte(0x10)){
            more.next();
            let result = expect_some!(
                Expression::parse(
                    next_or_err!(more, "Unexpected end of input: expected an expression.")?,
                    more
                )?,
                more,
                "an expression"
            )
            .map(|x| {
                Some(Rand {
                    count: Some(Box::new(x)),
                })
            });

            if more.peek() == Some(Token::OneByte(0x11)) {
                more.next();
            }

            result
        } else {
            Ok(Some(Rand { count: None }))
        }
    }
}

impl Reconstruct for Rand {
    fn reconstruct(&self, config: &crate::Config) -> Vec<Token> {
        if let Some(count) = &self.count {
            let inner = count.reconstruct(config);

            vec![Token::OneByte(0xAB), Token::OneByte(0x10)].into_iter().chain(inner).chain(iter::once(Token::OneByte(0x11))).collect::<Vec<_>>()
        } else {
            vec![Token::OneByte(0xAB)]
        }
    }
}
