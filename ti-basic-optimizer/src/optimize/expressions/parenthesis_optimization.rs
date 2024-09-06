//! # Parenthesis Optimization
//! Removing parentheses at the ends of lines is permissible in TI-BASIC, so we attempt to maximize
//! the number of parentheses that will be at the end of every line.

use std::mem;
use titokens::Token;

use crate::parse::commands::{Command, ControlFlow, Generic};
use crate::parse::components::{ListIndex, MatrixIndex, StoreTarget};
use crate::parse::{
    components::{Operand, Operator},
    expression::Expression,
};

impl Expression {
    /// Maximizes the number of parentheses which occur at the end of the line.
    ///
    /// Returns the number of parentheses that can be removed.
    pub fn optimize_parentheses(&mut self) -> u16 {
        /* there's some extra weirdness with lists/Ans & implicit mul that I'm not yet considering.
         * `Ans(1+X)` is a list access if Ans is a list. We avoid this with `(1+X)Ans` or `Ans*(1+X)`
         * If X has parentheses to be removed *and those parentheses get removed*, `Ans*(1+X)` is
         * shorter. Otherwise, `(1+X)Ans` is shorter or equivalent.
         *
         * One more note...
         * If `Ans(1+X)` is a list access or ambiguously written, we conservatively default to
         * parsing it as a list access- if it is actually an implicit multiplication then it will
         * always execute the same way as it did in the input. Defaulting to parsing as an
         * implicit multiplication may not result in the same code, because several passes depend on
         * multiplication being commutative.
         */

        /* more opt ideas: consider functions where changing the arguments does not change the result
         * (eg. min, max), and some functions on list literals where changing the order of the
         * list elements will not change the output.
         */

        /* even more opt ideas: stuff like /2 or /5 or /10 can be .5*, .2*, .1*
         * also x^^-1* if x is not 2, 5, 10, etc
         */

        match self {
            Expression::Operator(Operator::Binary(binop)) => {
                let mut right = binop.right.optimize_parentheses();

                if let Expression::Operator(Operator::Binary(right_binop)) = binop.right.as_ref() {
                    if binop.precedence() >= right_binop.precedence() {
                        right += 1;
                    }
                }

                if let Some(new_kind) = binop.opposite() {
                    let mut left = binop.left.optimize_parentheses();

                    if let Expression::Operator(Operator::Binary(left_binop)) = binop.left.as_ref()
                    {
                        if binop.precedence() >= left_binop.precedence() {
                            left += 1;
                        }
                    }

                    if left > right {
                        mem::swap(&mut binop.left, &mut binop.right);

                        binop.kind = new_kind;

                        left
                    } else {
                        right
                    }
                } else {
                    right
                }
            }

            Expression::Operator(Operator::FunctionCall(call)) => {
                if let Some(last) = call.arguments.last_mut() {
                    1 + last.optimize_parentheses()
                } else {
                    1
                }
            }

            Expression::Operator(Operator::Unary(unop)) => {
                if unop.kind == Token::OneByte(0xB0) {
                    if matches!(
                        unop.child.as_ref(),
                        Expression::Operator(Operator::Binary(_))
                    ) {
                        1 + unop.child.optimize_parentheses()
                    } else {
                        unop.child.optimize_parentheses()
                    }
                } else {
                    0
                }
            }

            Expression::Operand(Operand::ListAccess(ListIndex { index, .. })) => {
                1 + index.optimize_parentheses()
            }

            Expression::Operand(Operand::MatrixAccess(MatrixIndex { col, .. })) => {
                1 + col.optimize_parentheses()
            }

            Expression::Operand(Operand::StringLiteral(_) | Operand::ListLiteral(_)) => 1,

            _ => 0,
        }
    }

    /// Removes closing parenthesis, braces, and quotes from the provided reconstructed expression.
    ///
    /// Returns true if the expression ends in an unclosed string.
    pub fn strip_closing_parenthesis(expr: &mut Vec<Token>) -> bool {
        // a little tricky; `")))` should not have anything removed
        let mut length_guess = 0;
        let mut in_string = false;

        let mut unclosed_string = false;
        // 123"123)")

        for (idx, tok) in expr.iter().enumerate() {
            match tok {
                Token::OneByte(0x2A) => {
                    // "
                    in_string = !in_string;

                    if !in_string {
                        length_guess = idx - 1;
                        unclosed_string = true;
                    }
                }

                Token::OneByte(0x11 | 0x07 | 0x09) if !in_string => {
                    // ) ] }
                    // nothing
                }

                _ => {
                    length_guess = idx;

                    unclosed_string = in_string;
                }
            }
        }

        expr.truncate(length_guess + 1);

        unclosed_string
    }
}

impl Command {
    pub fn optimize_parentheses(&mut self) {
        match self {
            Command::Generic(Generic { arguments, .. }) => {
                if let Some(last) = arguments.last_mut() {
                    last.optimize_parentheses();
                }
            }

            Command::ControlFlow(control_flow) => {
                match control_flow {
                    ControlFlow::If(expr) | ControlFlow::While(expr) | ControlFlow::Repeat(expr) => {
                        expr.optimize_parentheses();
                    }
                    ControlFlow::For(for_loop) => {
                        if let Some(step) = &mut for_loop.step {
                            step.optimize_parentheses();
                        } else {
                            for_loop.end.optimize_parentheses();
                        }
                    }
                    ControlFlow::IsGt(is_ds) | ControlFlow::DsLt(is_ds) => {
                        is_ds.condition.optimize_parentheses();
                    }

                    _ => { /* no closing parentheses savings possible */ }
                }
            }

            Command::Expression(expr) => {
                expr.optimize_parentheses();
            }
            Command::Store(expr, target) => {
                expr.optimize_parentheses();
                match target {
                    StoreTarget::ListIndex(index) => {
                        index.index.optimize_parentheses();
                    }
                    StoreTarget::MatrixIndex(index) => {
                        index.col.optimize_parentheses();
                    }

                    _ => { /* no closing parentheses possible */ }
                }
            }

            _ => { /* no closing parentheses possible */ }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::{Parse, Reconstruct};
    use test_files::{load_test_data, test_version};
    use super::*;

    #[test]
    fn parenthesis_optimization() {
        let cases = [("1.txt", 3), ("2.txt", 1)];
        for (case_name, expected_savings) in cases {
            let mut tokens = load_test_data(&("/snippets/optimize/parentheses/maximization/".to_string() + case_name));
            let mut expr = Expression::parse(tokens.next().unwrap(), &mut tokens)
                .unwrap()
                .unwrap();

            let savings = expr.optimize_parentheses();
            assert_eq!(expected_savings, savings);

            let reconstructed = expr.reconstruct(&test_version().into());
            let mut optimized = reconstructed.clone();
            Expression::strip_closing_parenthesis(&mut optimized);

            assert_eq!(expected_savings, (reconstructed.len() - optimized.len()) as u16);
        }
    }

    #[test]
    fn strip_closing_parentheses() {
        for case in ["1.txt", "2.txt", "3.txt", "4.txt", "5.txt", "6.txt"] {
            let mut actual = load_test_data(&("/snippets/optimize/parentheses/stripping/before/".to_string() + case)).collect::<Vec<_>>();
            let expected = load_test_data(&("/snippets/optimize/parentheses/stripping/after/".to_string() + case)).collect::<Vec<_>>();
            Expression::strip_closing_parenthesis(&mut actual);

            assert_eq!(actual, expected);
        }
    }
}
