use titokens::{Token, Tokens};
use crate::parse::components::NumericVarName;
use crate::parse::expression::Expression;
use crate::parse::Parse;

// IsDs the real life?
pub struct IsDs {
    pub variable: NumericVarName,
    pub condition: Expression,
}

impl Parse for IsDs {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        todo!()
    }
}