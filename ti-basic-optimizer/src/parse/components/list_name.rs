use crate::error_reporting::LineReport;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens};

#[derive(Copy, Clone, Debug)]
pub enum ListName {
    Default(Token),
    /// Must match the TI-ASCII bytes for `[A-Zθ][A-Zθ0-9]{,4}`, and be zero
    /// filled at the end.
    Custom([u8; 5]),
}

impl Parse for ListName {
    fn parse(token: Token, tokens: &mut Tokens) -> Result<Option<Self>, LineReport> {
        match token {
            // 5Dxx, lists
            Token::TwoByte(0x5D, 0x00..=0x05) => Ok(Some(ListName::Default(token))),

            // EB, |L
            Token::OneByte(0xEB) => {
                let start_position = tokens.current_position() - 1;
                let mut name = [0_u8; 5];
                let mut index = 0;

                while let Some(token) = tokens.peek() {
                    // 0-indexed
                    if index >= 5 {
                        Err(LineReport::new(
                            start_position,
                            "List name has too many characters (max 5)",
                            None,
                        )
                        .with_span_label(
                            start_position..start_position + 7,
                            "This part is a valid list name.",
                        )
                        .with_label(tokens.current_position(), "The part starting here is not."))?;
                    }

                    if index == 0 && token.is_alpha() || index > 0 && token.is_alphanumeric() {
                        name[index] = token.byte();
                        index += 1;
                    } else {
                        break;
                    }
                }

                if index == 0 {
                    Err(LineReport::new(
                        start_position,
                        "Expected a list name.",
                        Some("List names start with a letter A-θ."),
                    ))?;
                }

                Ok(Some(ListName::Custom(name)))
            }
            _ => Ok(None),
        }
    }
}

impl Reconstruct for ListName {
    fn reconstruct(&self) -> Vec<Token> {
        match self {
            ListName::Default(tok) => vec![*tok],
            ListName::Custom(name) => name
                .iter()
                .filter(|&&x| (x > 0))
                .cloned()
                .map(Token::OneByte)
                .collect(),
        }
    }
}
