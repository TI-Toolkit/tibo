use crate::error_reporting::LineReport;
use crate::parse::commands::Command;
use crate::parse::Parse;
use titokens::{tokenizer, Token, Tokenizer, Tokens};

pub struct Program {
    lines: Vec<Command>,
}

impl Program {
    pub fn from_tokens(tokens: &mut Tokens, tokenizer: &Tokenizer) -> Self {
        match Program::parse(tokens) {
            Ok(prog) => prog,
            Err(report) => {
                let boundaries = tokens.stringify_with_boundaries(tokenizer);
                report.error(boundaries);

                if cfg!(test) {
                    panic!("Error thrown; aborting.");
                } else {
                    std::process::exit(1);
                }
            }
        }
    }

    fn parse(tokens: &mut Tokens) -> Result<Program, LineReport> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::{load_test_data, test_version};

    #[test]
    fn parses_newlines_correctly_with_strings() {
        let mut tokens = load_test_data("/snippets/parsing/strings/newline-stuff.txt");
        let mut program = Program::from_tokens(&mut tokens, &Tokenizer::new(test_version(), "en"));

        assert_eq!(program.lines.len(), 5);
    }

    #[test]
    fn skips_blank_lines() {
        let mut tokens = load_test_data("/snippets/parsing/ten-blank-lines.txt");
        let mut program = Program::from_tokens(&mut tokens, &Tokenizer::new(test_version(), "en"));

        assert_eq!(program.lines.len(), 0);
    }

    #[test]
    fn parse() {
        let mut tokens = load_test_data("/programs/bouncy_ball/raw.txt");
        let tokenizer = Tokenizer::new(test_version(), "en");
        let program = Program::from_tokens(&mut tokens, &tokenizer);
    }
}
