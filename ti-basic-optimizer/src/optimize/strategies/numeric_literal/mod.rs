use crate::optimize::strategies::Strategy;
use crate::parse::Reconstruct;
use crate::Config;
use tifloats::Float;
use titokens::Token;

mod color_constant;
mod fpart_with_exponent;
mod integer_with_exponent;
mod math_constant;
mod write_digits;

use color_constant::ColorConstant;
use fpart_with_exponent::FPartWithExponent;
use integer_with_exponent::IntegerWithExponent;
use math_constant::MathConstant;
use write_digits::WriteDigits;

impl Reconstruct for Float {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        let strategies: Vec<Box<dyn Strategy<Self>>> = vec![
            Box::new(WriteDigits::new(*self)),
            Box::new(ColorConstant::new(*self, &config.mrov)),
            Box::new(MathConstant::new(*self)),
            Box::new(IntegerWithExponent::new(*self)),
            Box::new(FPartWithExponent::new(*self)),
        ];

        strategies.reconstruct(config)
    }
}
