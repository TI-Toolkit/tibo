//! # Color Constant
//! When available, colors are substantially faster than writing out numerals, because looking up
//! the float value from a static memory location is less expensive than parsing two digits.

use crate::optimize::strategies::Strategy;
use crate::parse::Reconstruct;
use crate::Config;
use tifloats::{tifloat, Float};
use titokens::{Token, Version};

pub(super) struct ColorConstant {
    item: Float,
    version: Version,
}

impl ColorConstant {
    pub(crate) fn new(item: Float, version: &Version) -> Self {
        Self {
            item,
            version: version.clone(),
        }
    }
}

impl Strategy<Float> for ColorConstant {
    fn exists(&self) -> bool {
        (self.version >= *titokens::version::EARLIEST_COLOR)
            && (tifloat!(0x0010000000000000 * 10 ^ 1)..=tifloat!(0x0024000000000000 * 10 ^ 1))
                .contains(&self.item)
            && self.item.significant_figures().len() <= 2
    }

    fn size_cost(&self) -> Option<usize> {
        self.exists().then_some(2)
    }

    fn speed_cost(&self) -> Option<u32> {
        self.exists().then_some(5898)
    }
}

impl Reconstruct for ColorConstant {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        assert!(self.exists());

        let sig_figs = self.item.significant_figures();
        let lower_byte =
            (0x41 - 10) + sig_figs[0] * 10 + if sig_figs.len() == 2 { sig_figs[1] } else { 0 };
        assert!((0x41..=0x4F).contains(&lower_byte));

        vec![Token::TwoByte(0xEF, lower_byte)]
    }
}
