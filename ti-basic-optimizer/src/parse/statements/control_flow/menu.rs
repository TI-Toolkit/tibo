use crate::error_reporting::{expect_some, expect_tok, next_or_err, TokenReport};
use crate::parse::{
    expression::Expression, statements::control_flow::LabelName, Parse, Reconstruct,
};
use crate::Config;
use std::iter::once;
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub struct Menu {
    pub title: Expression,
    pub option_titles: Vec<Expression>,
    pub option_labels: Vec<LabelName>,
}

impl Parse for Menu {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        if token != Token::OneByte(0xE6) {
            return Ok(None);
        }

        let title = expect_some!(
            Expression::parse(next_or_err!(more, "Expected a Menu title")?, more)?,
            more,
            "Expected a Menu title"
        )?;

        let mut option_titles = vec![];
        let mut option_labels = vec![];

        expect_tok!(
            more,
            Token::OneByte(0x2B),
            "Menus must have at least one option."
        )?;

        let mut next = more.next().unwrap();

        while let Some(expr) = Expression::parse(next, more)? {
            option_titles.push(expr);

            expect_tok!(more, Token::OneByte(0x2B), "a comma")?; // ,

            option_labels.push(expect_some!(
                LabelName::parse(next_or_err!(more, "Expected a label name.")?, more)?,
                more,
                "a label name",
                "Each menu option must have a valid label."
            )?);

            match more.peek() {
                Some(Token::OneByte(0x2B)) => {
                    // ,
                    more.next();
                }

                Some(Token::OneByte(0x11)) => {
                    // )
                    more.next();
                    break;
                }

                Some(Token::OneByte(0x3E | 0x3F)) | None => break, // :, \n, EOF

                Some(_) => Err(TokenReport::new(
                    more.current_position() - 1,
                    "Unexpected character in Menu(",
                    Some("perhaps it's unimplemented?"),
                )
                .with_label(more.current_position() - 1, "here"))?,
            }

            next = more.next().unwrap();
        }

        Ok(Some(Menu {
            title,
            option_titles,
            option_labels,
        }))
    }
}

impl Reconstruct for Menu {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        once(Token::OneByte(0xE6))
            .chain(self.title.reconstruct(config))
            .chain(
                self.option_titles
                    .iter()
                    .zip(self.option_labels.iter())
                    .flat_map(|(title, label)| {
                        once(Token::OneByte(0x2B))
                            .chain(title.reconstruct(config))
                            .chain(once(Token::OneByte(0x2B)))
                            .chain(label.reconstruct(config))
                    }),
            )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::{load_test_data, test_version};

    #[test]
    fn parse() {
        let mut tokens = load_test_data("/snippets/parsing/statements/menu.txt");

        let menu = Menu::parse(tokens.next().unwrap(), &mut tokens)
            .ok()
            .flatten()
            .unwrap();

        assert_eq!(
            menu.option_labels,
            vec![LabelName(0x504C), LabelName(0x5345), LabelName(0x3000)]
        )
    }

    #[test]
    fn reconstruct() {
        let data = load_test_data("/snippets/parsing/statements/menu.txt");
        let mut tokens = data.clone();
        let menu = Menu::parse(tokens.next().unwrap(), &mut tokens)
            .unwrap()
            .unwrap();

        assert_eq!(
            menu.reconstruct(&test_version!().into()),
            data.collect::<Vec<_>>()
        );
    }
}
