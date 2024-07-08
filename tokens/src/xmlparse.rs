use serde::{Deserialize, Serialize};

use crate::Version;

#[derive(Debug, Clone, Deserialize)]
struct Tokens {
    #[serde(rename = "$value", default)]
    tokens: Vec<TokenUnion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TokenUnion {
    Token(Token),
    TwoByte(TwoByte),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Token {
    #[serde(rename = "@value")]
    value: String,
    version: Vec<TokenVersion>,
}

impl Token {
    pub(crate) fn at(&self, version: &Version, lang_code: &str) -> &Translation {
        let mut iter = self.version.iter();

        let first = iter.next().unwrap(); // better have at least one version :)
        let mut translation: &Translation = first
            .translation(lang_code)
            .unwrap_or_else(|| first.translation("en").unwrap());

        for v in iter {
            if v.since <= *version {
                if let Some(t) = v.translation(lang_code) {
                    translation = t;
                }
            }
        }

        translation
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenVersion {
    since: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<Version>,
    #[serde(default)]
    lang: Vec<Translation>,
}

impl TokenVersion {
    fn translation(&self, lang_code: &str) -> Option<&Translation> {
        self.lang.iter().find(|x| x.code == lang_code)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TwoByte {
    #[serde(rename = "@value")]
    value: String,
    token: Vec<Token>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Translation {
    /// use Token::at
    #[serde(rename = "@code")]
    code: String,

    #[serde(rename = "@ti-ascii")]
    pub(crate) ti_ascii: String,

    pub(crate) display: String,

    pub(crate) accessible: String,

    #[serde(default)]
    pub(crate) variant: Vec<String>,
}

lazy_static::lazy_static! {
    static ref PARSED: Tokens =
        quick_xml::de::from_str::<Tokens>(include_str!("tokens/8X.xml")).unwrap();
    pub(crate) static ref DATA: Vec<(crate::Token, &'static Token)> = PARSED.tokens
        .iter()
        .flat_map(|token_union: &TokenUnion| match token_union {
            TokenUnion::Token(tok) => {
                vec![(
                    crate::Token::OneByte(
                        u8::from_str_radix(tok.value.trim_start_matches('$'), 16).unwrap(),
                    ),
                    tok,
                )]
            }

            TokenUnion::TwoByte(two_byte) => two_byte
                .token
                .iter()
                .map(|tok| {
                    (
                        crate::Token::TwoByte(
                            u8::from_str_radix(two_byte.value.trim_start_matches('$'), 16).unwrap(),
                            u8::from_str_radix(tok.value.trim_start_matches('$'), 16).unwrap(),
                        ),
                        tok,
                    )
                })
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();
}
