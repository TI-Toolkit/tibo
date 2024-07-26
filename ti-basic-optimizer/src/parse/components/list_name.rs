use titokens::{Token, Tokens};

use crate::parse::Parse;

#[derive(Copy, Clone, Debug)]
pub enum ListName {
    // $5D00
    L1,
    // $5D01
    L2,
    // $5D02
    L3,
    // $5D03
    L4,
    // $5D04
    L5,
    // $5D05
    L6,

    Custom {
        /// Must match the TI-ASCII bytes for `[A-Zθ][A-Zθ0-9]{,4}`, and be zero
        /// filled at the end.
        name: [u8; 5],
    },

    Ans,
}

impl Parse for ListName {
    fn parse(token: Token, tokens: &mut Tokens) -> Option<Self> {
        match token {
            // 5Dxx, lists
            Token::TwoByte(0x5D, x) if x < 0x06 => match x {
                0x00 => Some(ListName::L1),
                0x01 => Some(ListName::L2),
                0x02 => Some(ListName::L3),
                0x03 => Some(ListName::L4),
                0x04 => Some(ListName::L5),
                0x05 => Some(ListName::L6),
                // # Safety
                // x < 0x06 condition above prevents this from being reachable.
                _ => unsafe { core::hint::unreachable_unchecked() },
            },

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
