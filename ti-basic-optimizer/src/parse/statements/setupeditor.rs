use crate::error_reporting::TokenReport;
use crate::parse::components::{ListName, DEFAULT_LISTS};
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens};

use crate::Config;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct SetUpEditor {
    pub lists: Vec<ListName>,
}

impl Parse for SetUpEditor {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        if token != Token::TwoByte(0xBB, 0x4A) {
            return Ok(None);
        }

        let statement_position = more.current_position();

        let mut lists = vec![];
        if matches!(more.peek(), Some(Token::OneByte(0x3E | 0x3F)) | None) {
            while let Some(next) = more.next() {
                if let Some(name) = ListName::parse(next, more)? {
                    lists.push(name);
                } else {
                    more.backtrack_once();
                    if let Some(name) = ListName::parse_custom_name(more)? {
                        lists.push(name);
                    } else {
                        Err(TokenReport::new(
                            more.current_position(),
                            "Expected a list name",
                            None,
                        ))?;
                    }
                }

                match more.peek() {
                    Some(Token::OneByte(0x2B)) => {
                        // ,
                        more.next();
                    }
                    Some(Token::OneByte(0x3E | 0x3F)) | None => break, // :, \n, EOF

                    Some(_) => Err(TokenReport::new(
                        more.current_position() - 1,
                        "Unexpected character in SetUpEditor",
                        None,
                    )
                    .with_label(statement_position, "This SetUpEditor.")
                    .with_label(more.current_position() - 1, "here"))?,
                }
            }
        } else {
            lists.extend(Vec::from(DEFAULT_LISTS))
        }

        Ok(Some(SetUpEditor { lists }))
    }
}

impl Reconstruct for SetUpEditor {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        let mut result = vec![Token::TwoByte(0xBB, 0x4A)];
        if self.lists.len() == 6 && DEFAULT_LISTS.iter().all(|x| self.lists.contains(x)) {
            return result;
        }

        result.extend(
            self.lists
                .iter()
                .map(|name| name.reconstruct_custom_name(config))
                .intersperse(vec![Token::OneByte(0x2B)])
                .flatten(),
        );

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use titokens::version;

    #[test]
    fn argless() {
        let mut tokens = Tokens::from_vec(vec![], None);
        let token = Token::TwoByte(0xBB, 0x4A);

        assert_eq!(
            SetUpEditor::parse(token, &mut tokens)
                .unwrap()
                .unwrap()
                .reconstruct(&version::LATEST.clone().into()),
            vec![token]
        );
    }
}
