use crate::error_reporting::{expect_some, expect_tok, next_or_err, LineReport};
use crate::parse::{commands::control_flow::LabelName, expression::Expression, Parse};
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub struct Menu {
    pub title: Expression,
    pub option_titles: Vec<Expression>,
    pub option_labels: Vec<LabelName>,
}

impl Parse for Menu {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
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

                Some(x) => Err(LineReport::new(
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::load_test_data;

    #[test]
    fn works() {
        let mut tokens = load_test_data("/snippets/parsing/commands/menu.txt");

        let menu = Menu::parse(tokens.next().unwrap(), &mut tokens)
            .ok()
            .flatten()
            .unwrap();

        assert_eq!(
            menu.option_labels,
            vec![LabelName(0x504C), LabelName(0x5345), LabelName(0x3000)]
        )
    }
}
