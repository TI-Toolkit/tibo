pub use tokenizer::Tokenizer;
pub use version::{Model, Version};

pub mod tokenizer;
mod version;

mod xmlparse;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    OneByte(u8),
    /// upper byte, lower byte
    TwoByte(u8, u8),
}

impl Token {
    #[must_use]
    pub fn is_eol(&self) -> bool {
        matches!(
            self,
            // EOF is for completeness, but we shouldn't encounter it.
            Token::OneByte(0x00 | 0x3E | 0x3F)
        )
    }

    /// Includes θ
    #[must_use]
    pub fn is_alpha(&self) -> bool {
        matches!(self, Token::OneByte(0x41..=0x5B))
    }

    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(self, Token::OneByte(0x30..=0x39))
    }

    #[must_use]
    pub fn is_alphanumeric(&self) -> bool {
        self.is_alpha() || self.is_numeric()
    }

    /// Returns the least-significant byte in the token. For one-byte tokens this
    /// is the whole token, but for two-byte tokens this is the second byte.
    #[must_use]
    pub fn byte(&self) -> u8 {
        match *self {
            Token::TwoByte(_, x) | Token::OneByte(x) => x,
        }
    }

    #[must_use]
    pub fn string_escaped(&self) -> String {
        match self {
            Token::OneByte(a) => format!("\\x{{{:0>2x}}}", a),
            Token::TwoByte(a, b) => format!("\\x{{{:0>2x}{:0>2x}}}", a, b),
        }
    }
}

impl From<Token> for u16 {
    fn from(value: Token) -> Self {
        match value {
            Token::OneByte(a) => a as u16,
            Token::TwoByte(a, b) => ((a as u16) << 8) | (b as u16),
        }
    }
}

#[derive(Clone)]
pub struct Tokens {
    tokens: Vec<Token>,
    pos: usize,
    version: Version,
}

impl Iterator for Tokens {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok.copied()
    }
}

impl Tokens {
    #[must_use]
    pub fn from_bytes(bytes: &[u8], version: Version) -> Self {
        let mut iter = bytes.iter();
        let mut tokens = vec![];

        while let Some(&first) = iter.next() {
            let token = match first {
                0x5C..=0x5E | 0x60..=0x63 | 0x7E | 0xAA | 0xBB | 0xEF => {
                    Token::TwoByte(first, *iter.next().unwrap())
                }

                _ => Token::OneByte(first),
            };

            tokens.push(token);
        }

        Tokens::from_vec(tokens, version)
    }

    #[must_use]
    pub fn from_vec(tokens: Vec<Token>, version: Version) -> Self {
        Tokens {
            tokens,
            pos: 0,
            version,
        }
    }

    #[must_use]
    pub fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).copied()
    }

    pub fn backtrack_once(&mut self) {
        self.pos -= 1;
    }

    #[must_use]
    pub fn current_position(&self) -> usize {
        self.pos
    }

    pub fn to_string(&self, tokenizer: &Tokenizer) -> (String, Vec<usize>) {
        tokenizer.stringify(&self.tokens)
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
}
