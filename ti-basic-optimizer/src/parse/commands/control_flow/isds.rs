use crate::error_reporting::{expect_some, expect_tok, next_or_err, LineReport};
use crate::parse::components::NumericVarName;
use crate::parse::expression::Expression;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

// IsDs the real life?
#[derive(Clone, Debug)]
pub struct IsDs {
    pub variable: NumericVarName,
    pub condition: Expression,
}

impl Parse for IsDs {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
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
    fn reconstruct(&self, version: &Version) -> Vec<Token> {
        let mut result = self.variable.reconstruct(version);
        result.push(Token::OneByte(0x2B));
        result.extend(self.condition.reconstruct(version));

        result
    }
}
