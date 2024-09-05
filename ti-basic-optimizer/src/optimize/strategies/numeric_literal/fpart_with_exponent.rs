//! # Decimal-with-Exponent Representation
//! Attempt to put the float into the form `.<mantissa>|E<exponent>`, where all the significant
//! figures are placed behind the decimal point. Even though the initial parsing of the decimal point
//! is slow, digits are parsed much faster after a decimal point. This is only rarely chosen, usually
//! when [Priority::Speed](crate::Priority::Speed) is selected.
//!
//! Example: `1234` becomes `.1234|E4`

use super::write_digits::WriteDigits;
use crate::optimize::strategies::numeric_literal::integer_with_exponent::IntegerWithExponent;
use crate::optimize::strategies::Strategy;
use crate::parse::Reconstruct;
use crate::Config;
use tifloats::Float;
use titokens::Token;

pub(super) struct FPartWithExponent {
    original: Float,
    adjusted: Float,
}

impl FPartWithExponent {
    fn adjust(item: &Float) -> Float {
        item.shift(-item.exponent() - 1)
    }

    pub fn new(item: Float) -> Self {
        Self {
            original: item,
            adjusted: Self::adjust(&item),
        }
    }
}

impl Strategy<Float> for FPartWithExponent {
    fn exists(&self) -> bool {
        (-99..=99).contains(&(self.original.exponent() - self.adjusted.exponent()))
    }

    fn size_cost(&self) -> Option<usize> {
        self.exists().then(|| {
            let negation_cost = if self.original.is_negative() { 1 } else { 0 };

            let sig_figs = self.original.significant_figures().len();
            let shift = self.original.exponent() - self.adjusted.exponent();

            let exponent_cost = match shift {
                0..=9 => 1,
                -9..=-1 | 10..=99 => 2,
                -99..=-10 => 3,
                _ => unreachable!(),
            };

            negation_cost + 1 + sig_figs + 1 + exponent_cost
        })
    }

    fn speed_cost(&self) -> Option<u32> {
        self.exists().then(|| {
            // WriteDigits always exists
            let mantissa_cost = WriteDigits::new(self.adjusted).speed_cost().unwrap();

            let required_shift = self.original.exponent() - self.adjusted.exponent();
            // IntegerWithExponent::exponent_speed_cost is always Some if FPartWithExponent exists
            let exponent_cost = IntegerWithExponent::exponent_speed_cost(required_shift).unwrap();

            mantissa_cost + exponent_cost
        })
    }
}

impl Reconstruct for FPartWithExponent {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        assert!(self.exists());

        let mut result = WriteDigits::new(self.adjusted).reconstruct(config);

        result.push(Token::OneByte(0x3B));

        let mut exponent = self.original.exponent() - self.adjusted.exponent();
        if exponent < 0 {
            result.push(Token::OneByte(0xB0));
            exponent = exponent.abs();
        }

        if exponent > 10 {
            result.push(Token::OneByte(0x30 + (exponent as u8 / 10)));
        }

        result.push(Token::OneByte(0x30 + (exponent as u8 % 10)));

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tifloats::tifloat;

    #[test]
    fn adjust() {
        let cases = [
            tifloat!(0x0010000000000000 * 10 ^ 1),
            tifloat!(0x0010000000000000 * 10 ^ 2),
            tifloat!(0x0011000000000000 * 10 ^ -1),
            tifloat!(0x0011100000000000 * 10 ^ -10),
            tifloat!(0x0010000000000000 * 10 ^ -11),
            tifloat!(0x0011000000000000 * 10 ^ -11),
        ];
        for case in &cases {
            assert_eq!(FPartWithExponent::adjust(case).exponent(), -1);
        }
    }

    #[test]
    fn speed_cost() {
        let version = &*titokens::version::LATEST;

        let cases = vec![
            (tifloat!(0x0010000000000000 * 10 ^ 1), 11635),
            (tifloat!(0x0010000000000000 * 10 ^ 2), 11635),
            (tifloat!(0x0011000000000000 * 10 ^ -1), 13502),
            (tifloat!(0x0011100000000000 * 10 ^ -10), 16526),
            (tifloat!(0x0010000000000000 * 10 ^ -11), 13924),
            (tifloat!(0x0011000000000000 * 10 ^ -11), 15791),
        ];

        for (item, expected) in cases {
            assert_eq!(FPartWithExponent::new(item).speed_cost(), Some(expected));
        }
    }
}
