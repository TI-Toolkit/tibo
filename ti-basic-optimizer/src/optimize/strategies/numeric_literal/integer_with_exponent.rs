//! # Decimal-with-Exponent Representation
//! Attempt to put the float into the form `<mantissa>|E<exponent>`, where all of the significant
//! figures are placed before the `|E`. This is usually substantially faster than writing every zero.

use super::write_digits::{WriteDigits, BASE_COST, DIGIT_COST, SHIFTING_COST};
use crate::optimize::strategies::Strategy;
use crate::parse::Reconstruct;
use crate::Config;
use tifloats::Float;
use titokens::Token;

// time to parse <x>
#[rustfmt::skip]
macro_rules! ttp {
    (1|E1) => {10621};
    (1|E~1) => {11699};
    (1|E11) => {11832};
    (1|E21) => {11893};
}

pub(super) const EXPONENT_DECADE_COST: u32 = ttp!(1 | E21) - ttp!(1 | E11);
pub(super) const EXPONENT_NEGATION_COST: u32 = ttp!(1|E~1) - ttp!(1 | E1);
pub(super) const EXPONENT_TENS_COST: u32 = ttp!(1 | E11) - ttp!(1 | E1) - EXPONENT_DECADE_COST;
pub(super) const EXPONENT_BASE_COST: u32 = ttp!(1 | E1) - BASE_COST - DIGIT_COST - SHIFTING_COST;

// todo: drop the significant figure when it is just 1
pub(super) struct IntegerWithExponent {
    original: Float,
    adjusted: Float,
}

impl IntegerWithExponent {
    fn adjust(item: &Float) -> Float {
        item.shift(-(item.exponent() - item.significant_figures().len() as i8 + 1))
    }

    /// Computes the speed cost due to just the |E part of a numeric literal.
    pub fn exponent_speed_cost(exponent: i8) -> Option<u32> {
        (-99..=99).contains(&exponent).then(|| {
            let base_cost = EXPONENT_BASE_COST;
            let neg_cost = if exponent < 0 {
                EXPONENT_NEGATION_COST
            } else {
                0
            };

            let decades = (exponent.unsigned_abs() / 10) as u32;
            let decade_cost = if decades != 0 {
                EXPONENT_TENS_COST + EXPONENT_DECADE_COST * decades
            } else {
                0
            };

            base_cost + neg_cost + decade_cost
        })
    }

    pub fn new(item: Float) -> Self {
        Self {
            original: item,
            adjusted: Self::adjust(&item),
        }
    }
}

impl Strategy<Float> for IntegerWithExponent {
    fn exists(&self) -> bool {
        (-99..=99).contains(&(self.original.exponent() - self.adjusted.exponent()))
    }

    fn size_cost(&self) -> Option<usize> {
        self.exists().then(|| {
            1 + if self.original.significant_figures() == vec![1] {
                0
            } else {
                self.original.significant_figures().len()
            } + match self.original.exponent() - self.adjusted.exponent() {
                0..=9 => 1,
                -9..=-1 | 10..=99 => 2,
                -99..=-10 => 3,
                _ => unreachable!(),
            }
        })
    }

    fn speed_cost(&self) -> Option<u32> {
        self.exists().then(|| {
            // WriteDigits always exists & self.exists iff the required adjustment is in range.
            WriteDigits::new(self.adjusted).speed_cost().unwrap()
                + Self::exponent_speed_cost(self.original.exponent() - self.adjusted.exponent())
                    .unwrap()
        })
    }
}

impl Reconstruct for IntegerWithExponent {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        let mut result = if self.original.significant_figures() == vec![1] {
            vec![]
        } else {
            WriteDigits::new(self.adjusted).reconstruct(config)
        };
        result.push(Token::OneByte(0x3B));

        let mut exponent = self.original.exponent() - self.adjusted.exponent();
        if exponent < 0 {
            result.push(Token::OneByte(0xB0));
            exponent = exponent.abs();
        }

        if exponent >= 10 {
            result.push(Token::OneByte(0x30 + exponent as u8 / 10));
        }

        result.push(Token::OneByte(0x30 + exponent as u8 % 10));

        result
    }
}
