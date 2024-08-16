use crate::parse::expression::Expression;
use crate::parse::Parse;
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub struct ForLoop {
    pub iterator: Expression,
    pub start: Expression,
    pub end: Expression,
    pub step: Option<Expression>,
}

impl Parse for ForLoop {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        if token != Token::OneByte(0xD3) {
            return None;
        }

        let iterator = Expression::parse(more.next().unwrap(), more).unwrap();
        assert_eq!(more.next(), Some(Token::OneByte(0x2B))); // ,
        let start = Expression::parse(more.next().unwrap(), more).unwrap();
        assert_eq!(more.next(), Some(Token::OneByte(0x2B)));
        let end = Expression::parse(more.next().unwrap(), more).unwrap();

        let mut step = None;

        if more.peek() == Some(Token::OneByte(0x2B)) {
            more.next();
            step = Some(Expression::parse(more.next().unwrap(), more).unwrap());
        }

        if more.peek() == Some(Token::OneByte(0x11)) {
            more.next();
        }

        Some(ForLoop {
            iterator,
            start,
            end,
            step,
        })
    }
}
