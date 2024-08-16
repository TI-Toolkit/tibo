use std::collections::{BTreeMap, Bound};
use std::fmt::{Display, Formatter};
use std::ops::{Range, RangeBounds};
use radix_trie::{Trie, TrieCommon};

use crate::{Token, Tokens, Version};

/// Immutable container for text with an extra layer of information stating where tokens start and
/// end.
#[derive(Clone)]
#[must_use]
pub struct TokenBoundaries {
    text: String,
    boundaries: Vec<usize>,
}

impl TokenBoundaries {
    /// Translate from a token index into a range of indices into the contained string representation.
    ///
    /// # Example
    /// ```rust
    /// # use titokens::{Model, Token, Tokenizer, Version};
    /// # let tokenizer = Tokenizer::new(Version { model: Model::TI84PCE, os_version: "5.3.0".to_string()},"en");
    /// let source = "sin(2pi)";
    /// let (tokens, boundaries) = tokenizer.tokenize(source).unwrap();
    /// assert_eq!(boundaries.single(2), 5..7);
    /// ```
    pub fn single(&self, idx: usize) -> Range<usize> {
        if idx == 0 {
            0..self.boundaries[idx]
        } else {
            self.boundaries[idx-1]..self.boundaries[idx]
        }
    }

    /// Translate from a range of token indices into a range of indices into the contained string
    /// representation.
    ///
    /// # Example
    /// ```rust
    /// # use titokens::{Model, Token, Tokenizer, Version};
    /// # let tokenizer = Tokenizer::new(Version { model: Model::TI84PCE, os_version: "5.3.0".to_string()},"en");
    /// let source = "sin(2pi)";
    /// let (tokens, boundaries) = tokenizer.tokenize(source).unwrap();
    /// assert_eq!(boundaries.range(..=2), 0..7);
    /// ```
    pub fn range<T>(&self, range: T) -> Range<usize> where T: RangeBounds<usize> {
        let start = match range.start_bound() {
            Bound::Included(x) => self.single(*x).start,
            Bound::Excluded(x) => self.single(*x).end,
            Bound::Unbounded => 0
        };

        let end = match range.end_bound() {
            Bound::Included(x) => self.single(*x).end,
            Bound::Excluded(x) => self.single(*x).start,
            Bound::Unbounded => *self.boundaries.last().unwrap_or(&0)
        };

        start..end
    }
}
impl Display for TokenBoundaries {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

pub struct Tokenizer {
    names: BTreeMap<crate::Token, String>,
    trie: Trie<String, crate::Token>,
    version: Version,
}

impl Tokenizer {
    pub fn new(version: Version, lang_code: &str) -> Self {
        let mut names = BTreeMap::new();
        let mut trie = Trie::new();

        crate::xmlparse::DATA.iter().for_each(|(key, value)| {
            names.insert(*key, value.at(&version, lang_code).display.clone());
            trie.insert(value.at(&version, lang_code).accessible.clone(), *key);
        });

        trie.insert("\r\n".to_string(), Token::OneByte(0x3F));

        Tokenizer {
            names,
            trie,
            version,
        }
    }

    pub fn tokenize(&self, text: &str) -> Result<(Tokens, TokenBoundaries), ()> {
        let mut pos = 0;
        let mut boundaries = vec![];
        let mut result = vec![];

        while pos < text.len() {
            if let Some(subtree) = self.trie.get_ancestor(&text[pos..]) {
                let (key, value) = (subtree.key().unwrap(), subtree.value().unwrap());

                pos += key.len();
                result.push(*value);
                boundaries.push(pos);
            } else {
                // todo: make gooder
                return Err(());
            }
        }

        Ok((
            Tokens::from_vec(result, Some(self.version.clone())),
            TokenBoundaries {
                text: text.to_string(),
                boundaries
            },
        ))
    }

    #[must_use]
    pub fn stringify(&self, tokens: &[Token]) -> TokenBoundaries {
        let strings = tokens
            .iter()
            .map(|tok| self.names.get(tok).unwrap_or(&tok.string_escaped()).clone())
            .collect::<Vec<String>>();

        let boundaries = strings
            .iter()
            .map(std::string::String::len)
            .scan(0_usize, |acc, length| {
                *acc += length;
                Some(*acc)
            })
            .collect::<Vec<usize>>();

        TokenBoundaries {
            text: strings.join(""),
            boundaries
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize() {
        let tokenizer = Tokenizer::new(
            Version {
                model: crate::Model::TI84PCE,
                os_version: "5.3.0".to_string(),
            },
            "en",
        );

        let (tokens, boundaries) = tokenizer.tokenize(&"randInt(X^^2,Y->A").unwrap();

        assert_eq!(
            tokens.collect::<Vec<_>>(),
            vec![
                Token::TwoByte(0xBB, 0x0A),
                Token::OneByte(0x58),
                Token::OneByte(0x0D),
                Token::OneByte(0x2B),
                Token::OneByte(0x59),
                Token::OneByte(0x04),
                Token::OneByte(0x41)
            ]
        );

        assert_eq!(boundaries.single(1), 8..9);
        assert_eq!(boundaries.range(2..=3), 9..13);
    }
}
