use crate::parse::{commands::control_flow::LabelName, expression::Expression, Parse};
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub struct Menu {
    pub title: Expression,
    pub option_titles: Vec<Expression>,
    pub option_labels: Vec<LabelName>,
}

impl Parse for Menu {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        if token != Token::OneByte(0xE6) {
            return None;
        }

        let title = Expression::parse(more.next()?, more).unwrap();
        let mut option_titles = vec![];
        let mut option_labels = vec![];

        let mut next = more.next().unwrap();

        while let Some(expr) = Expression::parse(next, more) {
            option_titles.push(expr);

            assert_eq!(more.next(), Some(Token::OneByte(0x2B))); // ,

            option_labels.push(LabelName::parse(more.next().unwrap(), more).unwrap());

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

                Some(x) => panic!("Unexpected token {:?} in menu.", x),
            }
        }

        Some(Menu {
            title,
            option_titles,
            option_labels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::load_test_data;

    #[test]
    fn works() {
        let mut tokens = load_test_data("/snippets/parsing/commands/menu.txt");

        let menu = Menu::parse(tokens.next().unwrap(), &mut tokens).unwrap();

        assert_eq!(
            menu.option_labels,
            vec![LabelName(0x504C), LabelName(0x5345), LabelName(0x3000)]
        )
    }
}
