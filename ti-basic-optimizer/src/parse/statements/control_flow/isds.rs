use crate::error_reporting::{expect_some, expect_tok, next_or_err, TokenReport};
use crate::parse::{components::NumericVarName, expression::Expression, Parse, Reconstruct};

use crate::Config;
use titokens::{Token, Tokens};

// IsDs the real life?
#[derive(Clone, Debug)]
pub struct IsDs {
    pub variable: NumericVarName,
    pub condition: Expression,
}

impl Parse for IsDs {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        if token == Token::OneByte(0xDA) || token == Token::OneByte(0xDB) {
            let variable = expect_some!(
                NumericVarName::parse(next_or_err!(more)?, more)?,
                more,
                "a numeric variable"
            )?;
            expect_tok!(more, Token::OneByte(0x2B), ",")?;
            let condition = expect_some!(
                Expression::parse(next_or_err!(more)?, more)?,
                more,
                "a condition"
            )?;

            Ok(Some(IsDs {
                variable,
                condition,
            }))
        } else {
            Ok(None)
        }
    }
}

impl Reconstruct for IsDs {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        let mut result = self.variable.reconstruct(config);
        result.push(Token::OneByte(0x2B));
        result.extend(self.condition.reconstruct(config));

        result
    }
}
