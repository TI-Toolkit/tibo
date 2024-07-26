use titokens::{Token, Tokens};
use crate::parse::expression::Expression;
use crate::parse::Parse;

#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub kind: Token,
    pub arguments: Vec<Expression>
}

impl Parse for FunctionCall {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        matches!(token.into(),
            0x12 | // Round
            0x13 | // PxlTest
            0x14 | // Augment
            0x15 | // RowSwap
            0x16 | // RowPlus
            0x17 | // TimesRow
            0x18 | // TimesRowPlus
            0x19 | // Max
            0x1A | // Min
            0x1B | // RtoPr
            0x1C | // RtoPTheta
            0x1D | // PtoRx
            0x1E | // PtoRy
            0x1F | // Median
            0x20 | // RandM
            0x21 | // Mean
            0x22 | // Solve
            0x23 | // Seq
            0x24 | // FnInt
            0x25 | // NDeriv
            0x27 | // FMin
            0x28 | // FMax
            0xB1 | // Int
            0xB2 | // Abs
            0xB3 | // Det
            0xB4 | // Identity
            0xB5 | // Dim
            0xB6 | // Sum
            0xB7 | // Prod
            0xB8 | // Not
            0xB9 | // IPart
            0xBA | // FPart
            0xBB00 | // Npv
            0xBB01 | // Irr
            0xBB02 | // Bal
            0xBB03 | // SigmaPrn
            0xBB04 | // SigmaInt
            0xBB07 | // Dbd
            0xBB08 | // Lcm
            0xBB09 | // Gcd
            0xBB0A | // RandInt
            0xBB0B | // RandBin
            0xBB0C | // Sub
            0xBB0D | // StdDev
            0xBB0E | // Variance
            0xBB0F | // InString
            0xBB10 | // Normalcdf
            0xBB11 | // InvNorm
            0xBB12 | // Tcdf
            0xBB13 | // ChiSquaredcdf
            0xBB14 | // Fcdf
            0xBB15 | // Binompdf
            0xBB16 | // Binomcdf
            0xBB17 | // Poissonpdf
            0xBB18 | // Poissoncdf
            0xBB19 | // Geometpdf
            0xBB1A | // Geometcdf
            0xBB1B | // Normalpdf
            0xBB1C | // Tpdf
            0xBB1D | // ChiSquaredpdf
            0xBB1E | // Fpdf
            0xBB1F | // RandNorm
            0xBB25 | // Conj
            0xBB26 | // Real
            0xBB27 | // Imag
            0xBB28 | // Angle
            0xBB29 | // CumSum
            0xBB2A | // Expr
            0xBB2B | // Length
            0xBB2C | // DeltaList
            0xBB2D | // Ref
            0xBB2E | // Rref
            0xBC | // Sqrt
            0xBD | // Cbrt
            0xBE | // Ln
            0xBF | // EPow
            0xC0 | // Log
            0xC1 | // TenPow
            0xC2 | // Sin
            0xC3 | // ASin
            0xC4 | // Cos
            0xC5 | // ACos
            0xC6 | // Tan
            0xC7 | // ATan
            0xC8 | // Sinh
            0xC9 | // ASinh
            0xCA | // Cosh
            0xCB | // ACosh
            0xCC | // Tanh
            0xCD | // ATanh
            0xEF00 | // SetDate
            0xEF01 | // SetTime
            0xEF02 | // CheckTmr
            0xEF03 | // SetDtFmt
            0xEF04 | // SetTmFmt
            0xEF05 | // TimeCnv
            0xEF06 | // DayOfWk
            0xEF07 | // GetDtStr
            0xEF08 | // GetTmStr
            0xEF13 | // InvT
            0xEF32 | // Remainder
            0xEF33 | // Summation
            0xEF34 | // LogBASE
            0xEF35 | // RandIntNoRep
            0xEF95 | // InvBinom
            0xEF97 | // ToString
            0xEF98 | // Eval
            0xEFA6 // Piecewise
        ).then(|| {
            let mut next = more.next().unwrap();

            let mut arguments = vec![];
            while let Some(expr) = Expression::parse(next, more) {
                arguments.push(expr);

                match more.peek() {
                    Some(Token::OneByte(0x2B)) => { // ,
                        more.next().unwrap();
                    }
                    Some(Token::OneByte(0x11)) => { // )
                        more.next().unwrap();
                        break
                    }
                    None => break, // :, \n, EOF

                    x => panic!("Unexpected token {:?} in function call.", x.unwrap())
                }

                next = more.next().unwrap();
            }

            FunctionCall {
                kind: token,
                arguments,
            }
        })
    }
}