use tifloats::{tifloat, Float};

use crate::error_reporting::LineReport;
use crate::parse::components::{string::TIString, Operand};
use crate::parse::Parse;
use titokens::{Token, Tokens};

pub struct Builder<'a> {
    tokens: &'a mut Tokens,

    is_negative: bool,
    exponent: i8,

    digits: Vec<u8>,
}

impl<'a> Builder<'a> {
    #[must_use]
    pub fn new(tokens: &'a mut Tokens) -> Self {
        Self {
            tokens,

            is_negative: false,
            exponent: 0,
            digits: vec![],
        }
    }

    pub fn parse(&mut self) -> Float {
        self.consume_zeros();

        match self.tokens.peek() {
            // leading decimal
            Some(Token::OneByte(0x3A)) => {
                self.tokens.next(); // skip decimal point
                let mut digits = self.digits();

                if digits.is_empty() {
                    panic!("Illegal decimal point; missing digits.");
                } else {
                    let leading_zeros = digits.iter().position(|&x| x != 0).unwrap();
                    digits.drain(..leading_zeros);
                    if leading_zeros >= 99 {
                        panic!("Floating point number too small.");
                    }

                    self.exponent = -(leading_zeros as i8) - 1; // 0 leading zeros is 10^-1

                    self.digits = digits;
                }
            }

            // Scientific E
            Some(Token::OneByte(0x3B)) => {
                self.tokens.next();
                // implied 1
                self.digits = vec![1];
                self.handle_scientific_notation();
            }

            Some(x) if x.is_numeric() => {
                let before_decimal = self.digits();

                match self.tokens.peek() {
                    Some(Token::TwoByte(0xEF, 0x2F)) => {
                        todo!()
                    }

                    _ => {}
                }

                if before_decimal.len() < 99 {
                    // #[allow(clippy::cast_lossless)] once it's stabilized
                    self.exponent = (before_decimal.len() - 1) as i8;
                } else {
                    panic!("Overflow: Too many digits.");
                }

                self.digits = before_decimal;

                if let Some(Token::OneByte(0x3A)) = self.tokens.peek() {
                    self.tokens.next();
                    let mut digits = self.digits();
                    self.digits.append(&mut digits);
                }

                if let Some(Token::OneByte(0x3B)) = self.tokens.peek() {
                    self.tokens.next();
                    self.handle_scientific_notation();
                }
            }

            Some(Token::TwoByte(_, _)) => {}

            _ => {}
        };

        self.finalize()
    }

    fn consume_zeros(&mut self) {
        while let Some(Token::OneByte(0x30)) = self.tokens.peek() {
            self.tokens.next();
        }
    }

    fn digits(&mut self) -> Vec<u8> {
        let digits = self
            .tokens
            .map_while(|token| token.is_numeric().then(|| token.byte() - 0x30))
            .collect::<Vec<_>>();
        self.tokens.backtrack_once();

        digits
    }

    fn handle_scientific_notation(&mut self) {
        let negative = if let Some(Token::OneByte(0xB0)) = self.tokens.peek() {
            self.tokens.next();

            true
        } else {
            false
        };

        let digits = self.digits();

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        match u8::try_from(digits.len()).unwrap_or(255) {
            0 => panic!("Missing required exponent"),
            1 => {
                if negative {
                    self.exponent -= digits[0] as i8;
                } else {
                    self.exponent += digits[0] as i8;
                }
            }
            2 => {
                if negative {
                    self.exponent -= (digits[0] * 10 + digits[1]) as i8;
                } else {
                    self.exponent += (digits[0] * 10 + digits[1]) as i8;
                }
            }
            3.. => panic!(
                "{}",
                if negative {
                    "E-99 is the lowest valid exponent"
                } else {
                    "E99 is the highest valid exponent."
                }
            ),
        }

        if let Some(Token::OneByte(0x3A)) = self.tokens.peek() {
            panic!("Unexpected decimal point.")
        }
    }

    fn finalize(&mut self) -> Float {
        let float = Float::new(
            self.is_negative,
            self.exponent,
            Float::mantissa_from(&self.digits),
        );

        if let Ok(ok) = float {
            ok
        } else {
            todo!()
        }
    }
}

impl Parse for tifloats::Float {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        match token {
            Token::OneByte(0x30..=0x3B) => {
                more.backtrack_once();
                let mut builder = Builder::new(more);
                Ok(Some(builder.parse()))
            }

            _ => Ok(None),
        }
    }
}

pub(crate) fn parse_constant(tok: Token, more: &mut Tokens) -> Option<Operand> {
    use super::Operand::NumericLiteral as NL;
    match tok {
        // pi
        Token::OneByte(0xAC) => Some(NL(tifloat!(0x0031415926535898 * 10 ^ 0))),
        // e
        Token::TwoByte(0xBB, 0x31) => Some(NL(tifloat!(0x0027182818284590 * 10 ^ 0))),

        // BLUE
        Token::TwoByte(0xEF, 0x41) => Some(NL(tifloat!(0x0010000000000000 * 10 ^ 1))),
        // RED, etc
        Token::TwoByte(0xEF, 0x42) => Some(NL(tifloat!(0x0011000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x43) => Some(NL(tifloat!(0x0012000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x44) => Some(NL(tifloat!(0x0013000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x45) => Some(NL(tifloat!(0x0014000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x46) => Some(NL(tifloat!(0x0015000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x47) => Some(NL(tifloat!(0x0016000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x48) => Some(NL(tifloat!(0x0017000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x49) => Some(NL(tifloat!(0x0018000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x4A) => Some(NL(tifloat!(0x0019000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x4B) => Some(NL(tifloat!(0x0020000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x4C) => Some(NL(tifloat!(0x0021000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x4D) => Some(NL(tifloat!(0x0022000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x4E) => Some(NL(tifloat!(0x0023000000000000 * 10 ^ 1))),
        Token::TwoByte(0xEF, 0x4F) => Some(NL(tifloat!(0x0024000000000000 * 10 ^ 1))),

        // LEFT
        Token::TwoByte(0xEF, 0x92) => {
            Some(Operand::StringLiteral(TIString::new(vec![Token::OneByte(
                0x30,
            )])))
        }
        // CENTER
        Token::TwoByte(0xEF, 0x93) => {
            Some(Operand::StringLiteral(TIString::new(vec![Token::OneByte(
                0x31,
            )])))
        }
        // RIGHT
        Token::TwoByte(0xEF, 0x94) => {
            Some(Operand::StringLiteral(TIString::new(vec![Token::OneByte(
                0x32,
            )])))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use tifloats::tifloat;

    use super::*;

    macro_rules! test_case {
        ($name:ident, $path:expr, $expected:expr) => {
            #[test]
            fn $name() {
                use test_files::load_test_data;
                let mut tokens = load_test_data($path);
                let mut builder = Builder::new(&mut tokens);

                assert_eq!(builder.parse(), $expected)
            }
        };
    }

    test_case!(
        one,
        "/snippets/parsing/numbers/one.txt",
        tifloat!(0x10000000000000 * 10 ^ 0)
    );

    test_case!(
        digits,
        "/snippets/parsing/numbers/digits.txt",
        tifloat!(0x12345678900000 * 10 ^ 9)
    );

    test_case!(
        exponents,
        "/snippets/parsing/numbers/9e99.txt",
        tifloat!(0x90000000000000 * 10 ^ 99)
    );

    test_case!(
        leading_zeros,
        "/snippets/parsing/numbers/leading-zeros.txt",
        tifloat!(0x50000500000000 * 10 ^ 0)
    );

    test_case!(
        leading_decimal,
        "/snippets/parsing/numbers/leading-decimal.txt",
        tifloat!(0x50000000000000 * 10 ^ -5)
    );

    test_case!(
        zero,
        "/snippets/parsing/numbers/zero.txt",
        tifloat!(0x00000000000000 * 10 ^ 0)
    );
}
