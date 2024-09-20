use itertools::Itertools;

use crate::error_reporting::{Report, TokenReport};
use crate::parse::commands::Command;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokenizer, Tokens, Version};

pub struct Program {
    pub lines: Vec<Command>,
}

impl Program {
    pub fn from_text(text: &str, version: Version) -> Self {
        let tokenizer = Tokenizer::new(version, "en");
        if let Ok((mut tokens, boundaries)) = tokenizer.tokenize(text) {
            match Program::parse(&mut tokens) {
                Ok(prog) => prog,
                Err(report) => {
                    report.report(boundaries);

                    if cfg!(test) {
                        panic!("Error thrown; aborting.");
                    } else {
                        std::process::exit(1);
                    }
                }
            }
        } else {
            eprintln!("Failed to tokenize input file.");
            if cfg!(test) {
                panic!("Error thrown; aborting.");
            } else {
                std::process::exit(1);
            }
        }
    }

    pub fn from_tokens(tokens: &mut Tokens, tokenizer: &Tokenizer) -> Self {
        match Program::parse(tokens) {
            Ok(prog) => prog,
            Err(report) => {
                let boundaries = tokens.stringify_with_boundaries(tokenizer);
                report.report(boundaries);

                if cfg!(test) {
                    panic!("Error thrown; aborting.");
                } else {
                    std::process::exit(1);
                }
            }
        }
    }

    fn parse(tokens: &mut Tokens) -> Result<Program, TokenReport> {
        let mut lines = vec![];

        let mut line_number = 1;
        while let Some(next) = tokens.next() {
            match next {
                Token::OneByte(0x3E) => continue,
                Token::OneByte(0x3F) => {
                    line_number += 1;
                    continue;
                }
                _ => {}
            }

            if let Some(command) = Command::parse(next, tokens)? {
                lines.push(command);
            }

            match tokens.peek() {
                Some(Token::OneByte(0x3E | 0x3F)) | None => continue,
                _ => {
                    let found_tokens = tokens
                        .take_while(|x| !matches!(x, Token::OneByte(0x3E | 0x3F)))
                        .map(|tok| tok.string_escaped())
                        .collect::<String>();
                    eprintln!("Warning: Line {line_number} contains tokens\n\n{found_tokens}\n\nwhich were unparsed\nThis will become an error in the future.");
                }
            }
        }

        Ok(Program { lines })
    }

    pub fn reconstruct(&self, config: &Config) -> Vec<Token> {
        // We choose to exclusively output 0x3F as a newline character because it means we never have
        // to worry about closing strings.
        self.lines
            .iter()
            .filter(|x| !matches!(x, Command::None))
            .map(|line| line.reconstruct(config))
            .intersperse(vec![Token::OneByte(0x3F)])
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::{load_test_data, test_tokenizer};

    #[test]
    fn parses_newlines_correctly_with_strings() {
        let mut tokens = load_test_data("/snippets/parsing/strings/newline-stuff.txt");
        let program = Program::from_tokens(&mut tokens, &test_tokenizer!());

        assert_eq!(program.lines.len(), 5);
    }

    #[test]
    fn skips_blank_lines() {
        let mut tokens = load_test_data("/snippets/parsing/ten-blank-lines.txt");
        let program = Program::from_tokens(&mut tokens, &test_tokenizer!());

        assert_eq!(program.lines.len(), 0);
    }

    /// TI-Toolkit defines "round-trip" as the following process:
    /// 1. Import original
    /// 2. Export to file A
    /// 3. Import file A
    /// 4. Export to file B
    /// 5. Then, check A == B
    mod round_trip {
        use super::*;
        use test_files::test_version;
        macro_rules! round_trip {
            ($name: ident, $path: expr, $debug: expr) => {
                #[test]
                fn $name() {
                    let mut original = load_test_data($path);
                    let tokenizer = test_tokenizer!();
                    let config = test_version!().into();
                    let original_program = Program::from_tokens(&mut original, &tokenizer);
                    let a = original_program.reconstruct(&config);
                    let a_program = Program::from_tokens(
                        &mut Tokens::from_vec(a.clone(), Some(test_version!())),
                        &tokenizer,
                    );
                    let b = a_program.reconstruct(&config);

                    if $debug {
                        dbg!(Tokens::from_vec(a.clone(), Some(test_version!()))
                            .to_string(&tokenizer));
                        dbg!(Tokens::from_vec(b.clone(), Some(test_version!()))
                            .to_string(&tokenizer));
                    }

                    assert_eq!(a, b);
                }
            };

            ($name: ident, $path: expr) => {
                round_trip!($name, $path, false);
            };
        }

        round_trip!(bouncy_ball, "/programs/bouncy_ball/raw.txt");
        round_trip!(stick_hero, "/programs/stick_hero/raw.txt");
    }
}
