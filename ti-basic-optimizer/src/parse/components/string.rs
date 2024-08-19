use itertools::Itertools;

use crate::error_reporting::LineReport;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

#[derive(Clone, Debug)]
pub struct TIString {
    data: Vec<Token>,
}

impl TIString {
    pub fn new(data: Vec<Token>) -> Self {
        TIString { data }
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
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        if token != Token::OneByte(0x2A) {
            return Ok(None);
        }

        let data: Vec<Token> = more
            .peeking_take_while(|tok| !matches!(tok, Token::OneByte(0x04 | 0x3F | 0x2A))) // ->, \n, "
            .collect();

        if let Some(Token::OneByte(0x2A)) = more.peek() {
            more.next();
        }

        Ok(Some(TIString::new(data)))
    }
}

impl Reconstruct for TIString {
    fn reconstruct(&self, version: Version) -> Vec<Token> {
        let mut tokens = vec![Token::OneByte(0x2A)];
        tokens.extend_from_slice(&self.data);
        tokens.push(Token::OneByte(0x2A));

        tokens
    }
}

mod tests {
    use super::*;

    #[test]
    fn parse_quote_terminated_string() {
        use test_files;

        let mut tokens =
            test_files::load_test_data("/snippets/parsing/strings/quote-terminated.txt");
        let result = TIString::parse(tokens.next().unwrap(), &mut tokens)
            .ok()
            .flatten()
            .unwrap();
        assert!(!result
            .data
            .iter()
            .any(|&t| matches!(t, Token::OneByte(0x2A))));
        assert_eq!(result.data.len(), 13);
    }
}
