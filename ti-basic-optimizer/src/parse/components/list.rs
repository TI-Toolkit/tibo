use crate::error_reporting::{next_or_err, LineReport};
use crate::parse::components::function_call::FunctionCall;
use crate::parse::expression::Expression;
use crate::parse::Parse;
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub struct TIList {
    pub entries: Vec<Expression>,
}

impl Parse for TIList {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        // {
        if token != Token::OneByte(0x08) {
            return Ok(None);
        }

        let mut next = next_or_err!(more, "Lists must have at least one element.")?;

        let mut entries = vec![];
        while let Some(expr) = Expression::parse(next, more)? {
            entries.push(expr);

            match more.peek() {
                Some(Token::OneByte(0x2B)) => {
                    // ,
                    more.next().unwrap();
                }
                Some(Token::OneByte(0x09)) => {
                    // }
                    more.next().unwrap();
                    break;
                }
                Some(Token::OneByte(0x04 | 0x3E | 0x3F)) | None => break, // -> :, \n, EOF

                x => panic!("Unexpected token {:?} in list definition.", x.unwrap()),
            }

            next = more.next().unwrap();
        }

        Ok(Some(TIList { entries }))
    }
}
