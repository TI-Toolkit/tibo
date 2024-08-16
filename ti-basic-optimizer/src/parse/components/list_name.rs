use titokens::{Token, Tokens};

use crate::parse::Parse;

#[derive(Copy, Clone, Debug)]
pub enum ListName {
    Default(Token),
    Custom {
        /// Must match the TI-ASCII bytes for `[A-Zθ][A-Zθ0-9]{,4}`, and be zero
        /// filled at the end.
        name: [u8; 5],
    },
}

impl Parse for ListName {
    fn parse(token: Token, tokens: &mut Tokens) -> Option<Self> {
        match token {
            // 5Dxx, lists
            Token::TwoByte(0x5D, 0x00..=0x05) => Some(ListName::Default(token)),

            // EB, |L
            Token::OneByte(0xEB) => {
                let mut name = [0_u8; 5];
                let mut index = 0;

                while let Some(token) = tokens.peek() {
                    if index >= 5 {
                        // syntax error:
                        // custom list name must be no longer than 5 chars

                        todo!()
                    }

                    if index == 0 && token.is_alpha() || index > 0 && token.is_alphanumeric() {
                        name[index] = token.byte();
                        index += 1;
                    } else {
                        break;
                    }
                }

                if index == 0 {
                    // syntax error:
                    // custom list name must start with an alpha character

                    todo!()
                }

                Some(ListName::Custom { name })
            }
            _ => None,
        }
    }
}
