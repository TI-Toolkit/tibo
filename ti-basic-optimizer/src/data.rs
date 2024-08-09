use std::fmt::{Debug, Display, Formatter};

use titokens::{Token, Tokens, Version};

#[derive(Clone)]
pub struct Line {
    pub tokens: Vec<Token>,
    pub parse: Option<LineParse>,
    original_line_number: usize,
}

#[derive(Debug, Clone)]
pub enum LineParse {
    Unparsed,
}

pub struct Program {
    pub lines: Vec<Line>,
    pub version: Version,
}

#[cfg(test)]
impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tokenizer = titokens::Tokenizer::new(test_files::test_version(), "en");
        self.lines.iter().map(|line| {
            let (line_text, _boundaries) = Tokens::from_vec(line.tokens.clone(), test_files::test_version()).to_string(&tokenizer);

            f.write_str(&format!("\n{:0>3}: {}", line.original_line_number, line_text))
        }).collect()
    }
}

impl Program {
    fn from_tokens(mut tokens: Tokens) -> Program {
        let mut in_string = false;
        let mut lines: Vec<Line> = vec![];

        let mut index = 1;
        while tokens.peek().is_some() {
            let mut ended_with_newline = false;

            let line = tokens.by_ref().take_while(|v| {
                match v {
                    Token::OneByte(0x2A) => { // "
                        in_string = !in_string;
                        true
                    }

                    Token::OneByte(0x04) => { // ->
                        in_string = false;
                        true
                    }

                    Token::OneByte(0x3F) => {
                        ended_with_newline = true;

                        in_string = false;
                        false
                    }

                    Token::OneByte(0x3E) if !in_string => {
                        in_string = false;
                        false
                    }

                    _ => {
                        true
                    }
                }
            }).collect::<Vec<Token>>();

            lines.push(Line {
                tokens: line,
                parse: None,
                original_line_number: index,

            });

            if ended_with_newline {
                index += 1;
            }
        }

        Program {
            lines,
            version: tokens.version().clone(),
        }
    }
}

impl From<Tokens> for Program {
    fn from(value: Tokens) -> Self {
        let mut tokens = value;

        Program::from_tokens(tokens)
    }
}

mod tests {
    use crate::data::Program;

    #[test]
    fn finds_newlines_correctly_with_strings() {
        use test_files::load_test_data;
        let tokens = load_test_data("/snippets/parsing/strings/newline-stuff.txt");

        assert_eq!(Program::from_tokens(tokens).to_string(), "\n001: Disp \"Hello:\n002: \"1:→Str1\n002: 1*2→A\n003: \"\"→Str2\n003: A+1→A");
    }

    #[test]
    fn finds_newlines_correctly_with_blank_lines() {
        use test_files::load_test_data;
        let tokens = load_test_data("/snippets/parsing/ten-blank-lines.txt");

        assert_eq!(Program::from_tokens(tokens).to_string(), "\n001: \n001: \n001: \n001: \n001: \n002: \n002: \n002: \n002: \n002: ");
    }
}