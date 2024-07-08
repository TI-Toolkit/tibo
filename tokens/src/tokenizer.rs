use std::collections::BTreeMap;

use radix_trie::{Trie, TrieCommon};

use crate::{Token, Tokens, Version};

// to-do: maybe make a dedicated struct for "strings-with-token-boundaries" that
// has nice methods for getting ranges etc?

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

    pub fn tokenize(&self, text: &str) -> Result<(Tokens, Vec<usize>), ()> {
        let mut pos = 0;
        let mut token_boundaries = vec![];
        let mut result = vec![];

        while pos < text.len() {
            if let Some(subtree) = self.trie.get_ancestor(&text[pos..]) {
                let (key, value) = (subtree.key().unwrap(), subtree.value().unwrap());

                pos += key.len();
                result.push(*value);
                token_boundaries.push(pos);
            } else {
                // todo: make gooder
                return Err(());
            }
        }

        Ok((
            Tokens::from_vec(result, self.version.clone()),
            token_boundaries,
        ))
    }

    #[must_use]
    pub fn stringify(&self, tokens: &[Token]) -> (String, Vec<usize>) {
        let strings = tokens
            .iter()
            .map(|tok| self.names.get(tok).unwrap_or(&tok.string_escaped()).clone())
            .collect::<Vec<String>>();

        let token_boundaries = strings
            .iter()
            .map(std::string::String::len)
            .scan(0_usize, |acc, length| {
                *acc += length;
                Some(*acc)
            })
            .collect::<Vec<usize>>();

        (strings.join(""), token_boundaries)
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

        let (tokens, boundaries) = tokenizer.tokenize(&"randInt(X^^2,Y").unwrap();

        assert_eq!(
            tokens.collect::<Vec<_>>(),
            vec![
                Token::TwoByte(0xBB, 0x0A),
                Token::OneByte(0x58),
                Token::OneByte(0x0D),
                Token::OneByte(0x2B),
                Token::OneByte(0x59),
            ]
        );

        assert_eq!(boundaries, vec![8, 9, 12, 13, 14]);
    }
}
