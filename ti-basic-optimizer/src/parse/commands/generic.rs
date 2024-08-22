use itertools::Itertools;

use crate::error_reporting::LineReport;
use crate::parse::expression::Expression;
use crate::parse::{Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

#[derive(Clone, Debug)]
pub struct Generic {
    pub kind: Token,
    pub arguments: Vec<Expression>,
}

impl Parse for Generic {
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
        if !Generic::recognize(token) {
            return Ok(None);
        }

        let mut command = Generic {
            kind: token,
            arguments: vec![],
        };

        let command_position = more.current_position() - 1;

        if Generic::accepts_parameters(token) {
            if !matches!(more.peek(), Some(Token::OneByte(0x3E | 0x3F)) | None) {
                let mut next = more.next().unwrap();
                while let Some(expr) = Expression::parse(next, more)? {
                    command.arguments.push(expr);

                    match more.peek() {
                        Some(Token::OneByte(0x2B)) => {
                            // ,
                            more.next();
                        }
                        Some(Token::OneByte(0x11)) if Generic::has_opening_parenthesis(token) => {
                            // )
                            more.next();
                            break;
                        }
                        Some(Token::OneByte(0x3E | 0x3F)) | None => break, // :, \n, EOF

                        Some(_) => Err(LineReport::new(
                            more.current_position() - 1,
                            "Unexpected character in command invocation",
                            Some("perhaps it's unimplemented?"),
                        )
                        .with_label(command_position, "This command.")
                        .with_label(more.current_position() - 1, "here"))?,
                    }

                    next = more.next().unwrap();
                }
            }
        }

        Ok(Some(command))
    }
}

impl Reconstruct for Generic {
    fn reconstruct(&self, version: &Version) -> Vec<Token> {
        use std::iter::once;

        once(self.kind)
            .chain(
                self.arguments
                    .iter()
                    .map(|x| x.reconstruct(version))
                    .intersperse(vec![Token::OneByte(0x2B)])
                    .flatten(),
            )
            .collect()
    }
}

impl Generic {
    fn recognize(token: Token) -> bool {
        matches!(
            token.into(),
            0x2E // CubicReg
            | 0x2F // QuartReg
            | 0x64 // Radian
            | 0x65 // Degree
            | 0x66 // Normal
            | 0x67 // Sci
            | 0x68 // Eng
            | 0x69 // Float
            | 0x73 // Fix
            | 0x74 // Horiz
            | 0x75 // Full
            | 0x76 // Func
            | 0x77 // Param
            | 0x78 // Polar
            | 0x79 // Seq
            | 0x7A // IndpntAuto
            | 0x7B // IndpntAsk
            | 0x7C // DependAuto
            | 0x7D // DependAsk
            | 0x7E00 // Sequential
            | 0x7E01 // Simul
            | 0x7E02 // PolarGC
            | 0x7E03 // RectGC
            | 0x7E04 // CoordOn
            | 0x7E05 // CoordOff
            | 0x7E06 // Thick
            | 0x7E07 // DotThick
            | 0x7E08 // AxesOn
            | 0x7E09 // AxesOff
            | 0x7E0A // GridDot
            | 0x7E0B // GridOff
            | 0x7E0C // LabelOn
            | 0x7E0D // LabelOff
            | 0x7E0E // Web
            | 0x7E0F // Time
            | 0x7E10 // UvAxes
            | 0x7E11 // VwAxes
            | 0x7E12 // UwAxes
            | 0x84 // Trace
            | 0x85 // ClrDraw
            | 0x86 // ZStandard
            | 0x87 // ZTrig
            | 0x88 // ZBox
            | 0x89 // ZoomIn
            | 0x8A // ZoomOut
            | 0x8B // ZSquare
            | 0x8C // ZInteger
            | 0x8D // ZPrevious
            | 0x8E // ZDecimal
            | 0x8F // ZoomStat
            | 0x90 // ZoomRcl
            | 0x91 // PrintScreen
            | 0x92 // ZoomSto
            | 0x93 // Text
            | 0x96 // FnOn
            | 0x97 // FnOff
            | 0x98 // StorePic
            | 0x99 // RecallPic
            | 0x9A // StoreGDB
            | 0x9B // RecallGDB
            | 0x9C // Line
            | 0x9D // Vertical
            | 0x9E // PtOn
            | 0x9F // PtOff
            | 0xA0 // PtChange
            | 0xA1 // PxlOn
            | 0xA2 // PxlOff
            | 0xA3 // PxlChange
            | 0xA4 // Shade
            | 0xA5 // Circle
            | 0xA6 // Horizontal
            | 0xA7 // Tangent
            | 0xA8 // DrawInv
            | 0xA9 // DrawF
            | 0xBB32 // SinReg
            | 0xBB33 // Logistic
            | 0xBB34 // LinRegTTest
            | 0xBB35 // ShadeNorm
            | 0xBB36 // ShadeT
            | 0xBB37 // ShadeChiSquared
            | 0xBB38 // ShadeF
            | 0xBB39 // MatrToList
            | 0xBB3A // ListToMatr
            | 0xBB3B // ZTest
            | 0xBB3C // TTest
            | 0xBB3D // TwoSampZTest
            | 0xBB3E // OnePropZTest
            | 0xBB3F // TwoPropZTest
            | 0xBB40 // ChiSquaredTest
            | 0xBB41 // ZInterval
            | 0xBB42 // TwoSampZInt
            | 0xBB43 // OnePropZInt
            | 0xBB44 // TwoPropZInt
            | 0xBB45 // GraphStyle
            | 0xBB46 // TwoSampTTest
            | 0xBB47 // TwoSampFTest
            | 0xBB48 // TInterval
            | 0xBB49 // TwoSampTInt
            | 0xBB4A // SetUpEditor
            | 0xBB4B // PmtEnd
            | 0xBB4C // PmtBgn
            | 0xBB4D // Real
            | 0xBB4E // REThetaI
            | 0xBB4F // APlusBI
            | 0xBB50 // ExprOn
            | 0xBB51 // ExprOff
            | 0xBB52 // ClrAllLists
            | 0xBB53 // GetCalc
            | 0xBB55 // EquToString
            | 0xBB56 // StringToEqu
            | 0xBB57 // ClearEntries
            | 0xBB58 // Select
            | 0xBB59 // ANOVA
            | 0xBB68 // Archive
            | 0xBB69 // UnArchive
            | 0xBBCE // GarbageCollect
            | 0xD8 // Pause
            | 0xDC // Input
            | 0xDD // Prompt
            | 0xDE // Disp
            | 0xDF // DispGraph
            | 0xE0 // Output
            | 0xE1 // ClrHome
            | 0xE2 // Fill
            | 0xE3 // SortA
            | 0xE4 // SortD
            | 0xE5 // DispTable
            | 0xE7 // Send
            | 0xE8 // Get
            | 0xE9 // PlotsOn
            | 0xEA // PlotsOff
            | 0xEC // Plot1
            | 0xED // Plot2
            | 0xEE // Plot3
            | 0xEF0F // ClockOff
            | 0xEF10 // ClockOn
            | 0xEF11 // OpenLib
            | 0xEF12 // ExecLib
            | 0xEF14 // ChiSquaredGOFTest
            | 0xEF15 // LinRegTInt
            | 0xEF16 // ManualFit
            | 0xEF17 // ZQuadrant1
            | 0xEF18 // ZFrac12
            | 0xEF19 // ZFrac13
            | 0xEF1A // ZFrac14
            | 0xEF1B // ZFrac15
            | 0xEF1C // ZFrac18
            | 0xEF1D // ZFrac110
            | 0xEF36 // MATHPRINT
            | 0xEF37 // CLASSIC
            | 0xEF38 // Nd
            | 0xEF39 // Und
            | 0xEF3A // AUTO
            | 0xEF3B // DEC
            | 0xEF3C // FRAC
            | 0xEF3D // FRACAPPROX
            | 0xEF5A // GridLine
            | 0xEF5B // BackgroundOn
            | 0xEF64 // BackgroundOff
            | 0xEF65 // GraphColor
            | 0xEF67 // TextColor
            | 0xEF6A // DetectAsymOn
            | 0xEF6B // DetectAsymOff
            | 0xEF6C // BorderColor
            | 0xEF74 // Thin
            | 0xEF75 // DotThin
            | 0xEF96 // Wait
            | 0xF2 // OneVarStats
            | 0xF3 // TwoVarStats
            | 0xF4 // LinRegABX
            | 0xF5 // ExpReg
            | 0xF6 // LnReg
            | 0xF7 // PwrReg
            | 0xF8 // MedMed
            | 0xF9 // QuadReg
            | 0xFA // ClrList
            | 0xFB // ClrTable
            | 0xFF // LinRegAXB
        )
    }

    fn accepts_parameters(token: Token) -> bool {
        matches!(token.into(),
              0x2Eu16..=0x2Fu16
            | 0x73
            | 0x7E08
            | 0x7E0A
            | 0x93
            | 0x96..=0xA9
            | 0xBB32..=0xBB4A
            | 0xBB53..=0xBB56
            | 0xBB58..=0xBB59
            | 0xBB68..=0xBB69
            | 0xBBCE
            | 0xD8
            | 0xDC..=0xDE
            | 0xE0
            | 0xE2..=0xE4
            | 0xE7..=0xEE
            | 0xEF11..=0xEF12
            | 0xEF14..=0xEF16
            | 0xEF5A..=0xEF5B
            | 0xEF65
            | 0xEF67
            | 0xEF6C
            | 0xEF96
            | 0xF3..=0xFB
            | 0xFF
        )
    }

    /// For commands which [accept parameters], does the command permit a closing parenthesis at the
    /// end of its line?
    fn has_opening_parenthesis(token: Token) -> bool {
        matches!(
            token.into(),
              0x93u16
            | 0x9C
            | 0x9E..=0xA5
            | 0xA7
            | 0xBB35..=0xBB40
            | 0xBB53
            | 0xBB55..=0xBB56
            | 0xBB58..=0xBB59
            | 0xE0
            | 0xE2..=0xE4
            | 0xE7..=0xE8
            | 0xEC..=0xEE
            | 0xEF11..=0xEF15
            | 0xEF65
            | 0xEF67
        )
    }
}
