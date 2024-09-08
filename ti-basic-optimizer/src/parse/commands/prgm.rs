use crate::error_reporting::TokenReport;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use std::iter::once;
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub struct ProgramName {
    pub name: Vec<Token>,
}

impl Parse for ProgramName {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        if token != Token::OneByte(0x5F) {
            return Ok(None);
        }

        let start_position = more.current_position() - 1;
        let mut name = vec![];

        while let Some(token) = more.next() {
            if (!name.is_empty() && token.is_alpha())
                || (name.is_empty() && token.is_alphanumeric())
            {
                if name.len() > 8 {
                    Err(TokenReport::new(
                        start_position,
                        "Program name has too many characters (max 8)",
                        None,
                    )
                    .with_span_label(
                        start_position..start_position + 9,
                        "This part is a valid program name.",
                    )
                    .with_label(more.current_position(), "The part starting here is not."))?;
                }

                name.push(token);
            } else {
                more.backtrack_once();
                break;
            }
        }

        if name.is_empty() {
            Err(TokenReport::new(
                start_position,
                "Expected a program name.",
                Some("Program names start with a letter A-θ."),
            ))?;
        }

        Ok(Some(ProgramName { name }))
    }
}

impl Reconstruct for ProgramName {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        once(Token::OneByte(0x5F))
            .chain(self.name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::load_test_data;

    #[test]
    fn parse() {
        let mut tokens = load_test_data("/snippets/parsing/commands/prgm.txt");

        let prgm = ProgramName::parse(tokens.next().unwrap(), &mut tokens)
            .unwrap()
            .unwrap();
        assert_eq!(
            prgm.name,
            vec![
                Token::OneByte(0x41),
                Token::OneByte(0x42),
                Token::OneByte(0x43),
                Token::OneByte(0x44),
                Token::OneByte(0x45),
                Token::OneByte(0x46),
                Token::OneByte(0x47),
                Token::OneByte(0x48)
            ]
        )
    }
}
