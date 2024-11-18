mod for_loop;
mod isds;
mod menu;

pub use {for_loop::ForLoop, isds::IsDs, menu::Menu};

use crate::error_reporting::{expect_some, next_or_err, TokenReport};
use crate::parse::{expression::Expression, Parse, Reconstruct};
use crate::Config;
use std::iter::once;
use titokens::{Token, Tokens};

/// Pseudo-label for the start of the program.
pub const START_LABEL: LabelName = LabelName::new_unchecked('s' as u8, Some('t' as u8));

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct LabelName(u16);
#[macro_export]
/// Call with one or two char literals for the letters of the label, or "theta" for theta.
///
/// Example:
/// ```
/// use tibo::label_name;
/// let lbl_12 = label_name!('1' '2');
/// let lbl_thetatheta = label_name!("theta" "theta");
macro_rules! label_name {
    ($first: tt $second: tt) => {
        $crate::parse::statements::LabelName::new(label_name!(<internal> $first), Some(label_name!(<internal> $second)))
    };

    ($first: tt) => {
        $crate::parse::statements::LabelName::new(label_name!(<internal> $first), None)
    };

    (<internal> "theta") => { 0x5B };
    (<internal> $x: literal) => {$x as u8};
}

impl LabelName {
    /// Construct a label with the given name.
    ///
    /// Example:
    /// ```
    /// # use tibo::parse::statements::LabelName;
    /// // corresponds to the label defined by `Lbl PL`
    /// let lbl_pl = LabelName::new('P' as u8, Some('L' as u8));
    /// // corresponds to the label defined by `Lbl theta`
    /// let lbl_theta = LabelName::new(1 + 'Z' as u8, None);
    /// assert_ne!(lbl_theta, lbl_pl);
    /// ```
    ///
    /// Consider using the [`label_name`] macro:
    /// ```
    /// use tibo::label_name;
    /// # use tibo::parse::statements::LabelName;
    /// # let lbl_pl = LabelName::new('P' as u8, Some('L' as u8));
    /// # let lbl_theta = LabelName::new(1 + 'Z' as u8, None);
    /// assert_eq!(lbl_pl, label_name!('P' 'L'));
    /// assert_eq!(lbl_theta, label_name!("theta"));
    /// ```
    pub fn new(first: u8, second: Option<u8>) -> Self {
        assert!(matches!(first, 0x41..=0x5B | 0x30..=0x39));
        if let Some(second) = second {
            assert!(matches!(second, 0x41..=0x5B | 0x30..=0x39))
        }

        LabelName((first as u16) << 8 | second.unwrap_or(0) as u16)
    }

    const fn new_unchecked(first: u8, second: Option<u8>) -> Self {
        LabelName(
            (first as u16) << 8
                | match second {
                    Some(x) => x as u16,
                    None => 0,
                },
        )
    }

    #[cfg(any(test, feature = "debug-tools"))]
    pub(crate) fn internal_id(&self) -> u16 {
        self.0
    }
}

impl Parse for LabelName {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        if !token.is_alphanumeric() {
            return Ok(None);
        }

        let mut data = (token.byte() as u16) << 8;

        if let Some(tok) = more.peek() {
            if tok.is_alphanumeric() {
                more.next();
                data |= tok.byte() as u16;
            }
        }

        Ok(Some(LabelName(data)))
    }
}

impl Reconstruct for LabelName {
    fn reconstruct(&self, _config: &Config) -> Vec<Token> {
        let mut data = vec![Token::OneByte((self.0 >> 8) as u8)];
        if self.0 & 0xFF != 0 {
            data.push(Token::OneByte((self.0 & 0xFF) as u8));
        }

        data
    }
}

#[derive(Clone, Debug)]
pub enum ControlFlow {
    If(Expression),
    Then,
    Else,
    While(Expression),
    Repeat(Expression),
    For(ForLoop),
    End,
    Return,
    Lbl(LabelName),
    Goto(LabelName),
    Stop,
    IsGt(IsDs),
    DsLt(IsDs),
    Menu(Menu),
}

impl Parse for ControlFlow {
    #[rustfmt::skip]
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        use ControlFlow as CF;
        use Expression as Expr;

        match token {
            Token::OneByte(0xCE) => Ok(Some(CF::If(expect_some!(Expr::parse(next_or_err!(more)?, more)?, more, "a condition")?))),
            Token::OneByte(0xCF) => Ok(Some(CF::Then)),
            Token::OneByte(0xD0) => Ok(Some(CF::Else)),
            Token::OneByte(0xD1) => Ok(Some(CF::While(expect_some!(Expr::parse(next_or_err!(more)?, more)?, more, "a loop condition")?))),
            Token::OneByte(0xD2) => Ok(Some(CF::Repeat(expect_some!(Expr::parse(next_or_err!(more)?, more)?, more, "a loop condition")?))),
            Token::OneByte(0xD3) => Ok(Some(CF::For(expect_some!(ForLoop::parse(token, more)?, more, "a for statement")?))),
            Token::OneByte(0xD4) => Ok(Some(CF::End)),
            Token::OneByte(0xD5) => Ok(Some(CF::Return)),
            Token::OneByte(0xD6) => Ok(Some(CF::Lbl(expect_some!(LabelName::parse(next_or_err!(more)?, more)?, more, "a label", "All Lbls must be followed by one or two numbers or letters.")?, ))),
            Token::OneByte(0xD7) => Ok(Some(CF::Goto(expect_some!(LabelName::parse(next_or_err!(more)?, more)?, more, "a label", "All Gotos must be followed by one or two numbers or letters.")?,))),
            Token::OneByte(0xD9) => Ok(Some(CF::Stop)),
            Token::OneByte(0xDA) => Ok(Some(CF::IsGt(expect_some!(IsDs::parse(token, more)?, more, "Is<( statement")?))),
            Token::OneByte(0xDB) => Ok(Some(CF::DsLt(expect_some!(IsDs::parse(token, more)?, more, "Ds>( statement")?))),
            Token::OneByte(0xE6) => Ok(Some(CF::Menu(expect_some!(Menu::parse(token, more)?, more, "Menu(")?))),
            _ => Ok(None),
        }
    }
}

impl Reconstruct for ControlFlow {
    #[rustfmt::skip]
    fn reconstruct(&self, version: &Config) -> Vec<Token> {
        match self {
            ControlFlow::If(cond) => once(Token::OneByte(0xCE)).chain(cond.reconstruct(version)).collect(),
            ControlFlow::Then => vec![Token::OneByte(0xCF)],
            ControlFlow::Else => vec![Token::OneByte(0xD0)],
            ControlFlow::While(cond) => once(Token::OneByte(0xD1)).chain(cond.reconstruct(version)).collect(),
            ControlFlow::Repeat(cond) => once(Token::OneByte(0xD2)).chain(cond.reconstruct(version)).collect(),
            ControlFlow::For(for_loop) => for_loop.reconstruct(version),
            ControlFlow::End => vec![Token::OneByte(0xD4)],
            ControlFlow::Return => vec![Token::OneByte(0xD5)],
            ControlFlow::Lbl(label) => once(Token::OneByte(0xD6)).chain(label.reconstruct(version)).collect(),
            ControlFlow::Goto(label) => once(Token::OneByte(0xD7)).chain(label.reconstruct(version)).collect(),
            ControlFlow::Stop => vec![Token::OneByte(0xD9)],
            ControlFlow::IsGt(isds) => once(Token::OneByte(0xDA)).chain(isds.reconstruct(version)).collect(),
            ControlFlow::DsLt(isds) => once(Token::OneByte(0xDB)).chain(isds.reconstruct(version)).collect(),
            ControlFlow::Menu(menu) => menu.reconstruct(version),
        }
    }
}
