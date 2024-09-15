/// Write the number digit by digit in the program.
///
/// This strategy is extensively documented in
/// [my first post on writing numbers](https://www.cemetech.net/forum/viewtopic.php?p=308266#308266).
use crate::optimize::strategies::Strategy;
use crate::parse::Reconstruct;
use crate::Config;
use std::cmp::max;
use tifloats::Float;
use titokens::Token;

/// time to parse `x`
#[rustfmt::skip]
macro_rules! ttp {
    (1) => {7075};
    (.1) => {8089};
    (11) => {9051};
    (10) => {9113};
    (.01) => {9511};
    (.11) => {9956};
    (111) => {11106};
}

pub(super) const DIGIT_COST: u32 = ttp!(11) - ttp!(1);
pub(super) const FRAC_DIGIT_COST: u32 = ttp!(.11) - ttp!(.1);
pub(super) const SHIFTING_COST: u32 = (ttp!(111) - ttp!(11)) - DIGIT_COST;
pub(super) const BASE_COST: u32 = ttp!(1) - DIGIT_COST - SHIFTING_COST;
pub(super) const ZERO_SIGFIG_COST: u32 = ttp!(10) - ttp!(11);
pub(super) const FRAC_LEADING_ZERO_COST: u32 = ttp!(.01) - ttp!(.1);
pub(super) const DECIMAL_POINT_COST: u32 = ttp!(.1) - BASE_COST - FRAC_DIGIT_COST - SHIFTING_COST;

pub(super) struct WriteDigits {
    item: Float,
}

impl WriteDigits {
    pub fn new(item: Float) -> Self {
        Self { item }
    }
}

impl Strategy<Float> for WriteDigits {
    fn exists(&self) -> bool {
        true
    }

    fn size_cost(&self) -> Option<usize> {
        self.exists().then(|| {
            1 + max(
                self.item.exponent().unsigned_abs() as usize,
                self.item.significant_figures().len(),
            ) + self.item.is_negative() as usize
        })
    }

    fn speed_cost(&self) -> Option<u32> {
        self.exists().then(|| {
            let exponent = self.item.exponent();
            /* I deliberately glossed over significant figures in my post & define them there as
             * "all the digits after the first nonzero digit" though this definition is not
             * consistently applied.
             * this is inefficient in practice; tifloatslib's significant_figures just gives the
             * digits between the first and last nonzero digits, inclusive. We compute the cost with
             * of the trailing or leading zero digits with a closed form expression.
             */
            let digits = self.item.significant_figures();

            let mut clock_cycles = BASE_COST;

            if exponent < 0 {
                clock_cycles += FRAC_LEADING_ZERO_COST * (exponent.unsigned_abs() as u32 - 1);
            } else if (digits.len() as i8) < exponent + 1 {
                let trailing_zero_count = exponent.unsigned_abs() as u32 + 1 - digits.len() as u32;
                clock_cycles += (DIGIT_COST + ZERO_SIGFIG_COST) * trailing_zero_count
                    + SHIFTING_COST * (((1 - digits.len() as u32 % 2) + trailing_zero_count) / 2);
            }

            if digits.len() as i8 > exponent + 1 {
                clock_cycles += DECIMAL_POINT_COST;
            }

            // this could definitely be described in a more rusty way with fold, but I'd rather this
            // was obviously a direct translation of my posted python code.
            for (index, digit) in digits.iter().enumerate() {
                if index as i8 > exponent {
                    clock_cycles += FRAC_DIGIT_COST;
                } else {
                    clock_cycles += DIGIT_COST;
                }

                if index % 2 == 0 {
                    clock_cycles += SHIFTING_COST;
                }

                if *digit == 0 {
                    clock_cycles += ZERO_SIGFIG_COST;
                }
            }

            clock_cycles
        })
    }
}

impl Reconstruct for WriteDigits {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        let sig_figs = self.item.significant_figures();

        let exponent = self.item.exponent();

        // this underestimates in the negative exponent case by the number of sig figs, but it's not
        // too far off usually
        let mut result = Vec::with_capacity(2 + exponent.unsigned_abs() as usize);

        if self.item.is_negative() {
            result.push(Token::OneByte(0xB0))
        }

        if exponent < 0 {
            result.push(Token::OneByte(0x3A));
            // need |exponent|-1 zeros
            for i in 0..exponent.abs() - 1 {
                result.push(Token::OneByte(0x30));
            }
        }

        result.extend(
            sig_figs
                .iter()
                .map(|x| Token::OneByte(0x30 + x))
                .collect::<Vec<_>>(),
        );

        if exponent >= 0 {
            // eg. 12 has sigfigs=2, exponent=1, does not need decimal point
            // eg. 1.5 has sigfigs=2, exponent=0, needs decimal point
            if sig_figs.len() > 1 + exponent as usize {
                result.insert(
                    result.len() + 1 + exponent as usize - sig_figs.len(),
                    Token::OneByte(0x3A),
                );
            } else if exponent as usize >= sig_figs.len() {
                // need exponent+1-sigfigs zeros
                // eg. 10=1.0 * 10^1 (has sigfigs=1, exponent=1, zeros=1)
                let zeros = exponent as usize + 1 - sig_figs.len();

                for i in 0..zeros {
                    result.push(Token::OneByte(0x30));
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tifloats::tifloat;

    #[test]
    fn speed_cost() {
        let cases = [
            (tifloat!(0x0010000000000000 * 10 ^ 6), 19540),
            (tifloat!(0x0010000000000000 * 10 ^ 7), 21578),
            (tifloat!(0x0011110000000000 * 10 ^ -2), 15191),
            (tifloat!(0x0011110000000000 * 10 ^ 2), 14096),
        ];

        for (case, expected) in cases {
            assert_eq!(WriteDigits::new(case).speed_cost(), Some(expected))
        }
    }
}
