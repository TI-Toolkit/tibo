use crate::parse::components::NumericVarName;
use crate::parse::expression::Expression;
use crate::parse::Parse;
use titokens::{Token, Tokens};

// IsDs the real life?
#[derive(Clone, Debug)]
pub struct IsDs {
    pub variable: NumericVarName,
    pub condition: Expression,
}

impl Parse for IsDs {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        (token == Token::OneByte(0xDA) || token == Token::OneByte(0xDB)).then(|| {
            let variable = NumericVarName::parse(more.next().unwrap(), more).unwrap();
            assert_eq!(more.next(), Some(Token::OneByte(0x2B)));
            let condition = Expression::parse(more.next().unwrap(), more).unwrap();

            IsDs {
                variable,
                condition,
            }
        })
    }
}
