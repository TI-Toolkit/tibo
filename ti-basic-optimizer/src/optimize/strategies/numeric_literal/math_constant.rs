//! # Math Constant
//! When available, using `pi` or `e` is substantially faster and smaller than writing out the
//! digits.

use crate::optimize::strategies::Strategy;
use crate::parse::Reconstruct;
use crate::Config;
use tifloats::{tifloat, Float};
use titokens::Token;

pub(super) struct MathConstant {
    item: Float,

    kind: Option<Token>,
}

impl MathConstant {
    pub fn kind(item: Float) -> Option<Token> {
        if item == tifloat!(0x0031415926535898 * 10 ^ 0) {
            Some(Token::OneByte(0xAC))
        } else if item == tifloat!(0x0027182818284590 * 10 ^ 0) {
            Some(Token::TwoByte(0xBB, 0x31))
        } else {
            None
        }
    }
}

impl MathConstant {
    pub fn new(item: Float) -> Self {
        Self {
            item,
            kind: Self::kind(item),
        }
    }
}

impl Strategy<Float> for MathConstant {
    fn exists(&self) -> bool {
        self.kind.is_some()
    }

    fn size_cost(&self) -> Option<usize> {
        self.exists().then(|| match self.kind.unwrap() {
            Token::OneByte(_) => 1,
            Token::TwoByte(_, _) => 2,
        })
    }

    fn speed_cost(&self) -> Option<u32> {
        self.exists().then(|| match self.kind.unwrap() {
            Token::OneByte(0xAC) => 4819,
            Token::TwoByte(0xBB, 0x31) => 4784,
            _ => unreachable!(),
        })
    }
}

impl Reconstruct for MathConstant {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        assert!(self.exists());

        self.kind.into_iter().collect()
    }
}
