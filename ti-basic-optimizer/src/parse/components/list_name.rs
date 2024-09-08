use crate::error_reporting::TokenReport;
use crate::parse::components::NumericVarName;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

pub const DEFAULT_LISTS: [ListName; 6] = [
    ListName::Default(Token::TwoByte(0x5D, 0x00)),
    ListName::Default(Token::TwoByte(0x5D, 0x01)),
    ListName::Default(Token::TwoByte(0x5D, 0x02)),
    ListName::Default(Token::TwoByte(0x5D, 0x03)),
    ListName::Default(Token::TwoByte(0x5D, 0x04)),
    ListName::Default(Token::TwoByte(0x5D, 0x05)),
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ListName {
    /// L1, L2, ..., L6
    Default(Token),
    /// Must match the TI-ASCII bytes for `[A-Zθ][A-Zθ0-9]{,4}`, and be zero
    /// filled at the end.
    Custom([u8; 5]),
}

impl TryFrom<NumericVarName> for ListName {
    type Error = ();

    fn try_from(value: NumericVarName) -> Result<Self, Self::Error> {
        match value.0 {
            Token::OneByte(x) => Ok(ListName::Custom([x, 0, 0, 0, 0])),
            _ => Err(()),
        }
    }
}

impl ListName {
    /// Parse the up-to-5-character custom list name, without the beginning |L.
    pub fn parse_custom_name(tokens: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        let start_position = tokens.current_position() - 1;
        let mut name = [0_u8; 5];
        let mut index = 0;

        while let Some(token) = tokens.next() {
            if (index == 0 && token.is_alpha()) || (index > 0 && token.is_alphanumeric()) {
                // 0-indexed
                if index >= 5 {
                    return Err(TokenReport::new(
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

                name[index] = token.byte();
                index += 1;
            } else {
                tokens.backtrack_once();
                break;
            }
        }

        if index == 0 {
            Err(TokenReport::new(
                start_position,
                "Expected a list name.",
                Some("List names start with a letter A-θ."),
            ))?;
        }

        Ok(Some(ListName::Custom(name)))
    }

    /// Reconstruct the up-to-5-character custom list name, without the beginning |L, or the
    /// original token if this is a default list.
    pub fn reconstruct_custom_name(&self, config: &Config) -> Vec<Token> {
        match self {
            ListName::Default(_) => self.reconstruct(config),
            ListName::Custom(name) => name
                .iter()
                .filter(|&&x| (x > 0))
                .cloned()
                .map(Token::OneByte)
                .collect(),
        }
    }
}

impl Parse for ListName {
    fn parse(token: Token, tokens: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        match token {
            // 5Dxx, lists
            Token::TwoByte(0x5D, 0x00..=0x05) => Ok(Some(ListName::Default(token))),

            // EB, |L
            Token::OneByte(0xEB) => ListName::parse_custom_name(tokens),
            _ => Ok(None),
        }
    }
}

impl Reconstruct for ListName {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        match self {
            ListName::Default(tok) => vec![*tok],
            ListName::Custom(name) => std::iter::once(Token::OneByte(0xEB))
                .chain(
                    name.iter()
                        .filter(|&&x| (x > 0))
                        .cloned()
                        .map(Token::OneByte),
                )
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::test_version;

    #[test]
    fn parse() {
        let name = vec![
            Token::OneByte(0xEB),
            Token::OneByte(0x41),
            Token::OneByte(0x42),
            Token::OneByte(0x43),
            Token::OneByte(0x44),
            Token::OneByte(0x45),
        ];
        let mut tokens: Tokens = Tokens::from_vec(name.clone(), None);

        let parsed = ListName::parse(tokens.next().unwrap(), &mut tokens)
            .unwrap()
            .unwrap();
        assert_eq!(parsed.reconstruct(&test_version().into()), name);
    }
}
