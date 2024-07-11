use itertools::Itertools;

use titokens::{Token, Tokens};

use crate::parse::Parse;

#[derive(Clone, Debug)]
pub struct TIString {
    data: Vec<Token>,
}

impl TIString {
    pub fn new(data: Vec<Token>) -> Self {
        TIString {
            data,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the number of `Token`s in the `TIString`.
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Parse for TIString {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        matches!(token, Token::OneByte(0x2A)).then(|| {
            let mut data: Vec<Token> = more.peeking_take_while(|tok| !matches!(tok, Token::OneByte(0x04 | 0x3F | 0x2A))).collect(); // ->, \n, "
            match more.peek() {
                Some(Token::OneByte(0x2A)) => { more.next(); }
                _ => {}
            }

            TIString::new(data)
        })
    }
}

mod tests {
    use super::*;

    #[test]
    fn parse_quote_terminated_string() {
        use test_files;

        let mut tokens = test_files::load_test_data("/snippets/parsing/strings/quote-terminated.txt");
        let result = TIString::parse(tokens.next().unwrap(), &mut tokens).unwrap();
        assert!(!result.data.iter().any(|&t| matches!(t, Token::OneByte(0x2A))));
        assert_eq!(result.data.len(), 13);
    }
}