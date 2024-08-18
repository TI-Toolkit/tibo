//! # Parenthesis Optimization
//! Removing parentheses at the ends of lines is permissible in TI-BASIC, so we attempt to maximize
//! the number of parentheses that will be at the end of every line.

use std::mem;
use titokens::Token;

use crate::parse::{components::Operator, expression::Expression};

impl Expression {
    /// Maximizes the number of parentheses which occur at the end of the line.
    ///
    /// Returns the number of parentheses that can be removed.
    pub fn optimize_parentheses(&mut self) -> u16 {
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

            _=> 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::Parse;
    use test_files::load_test_data;

    use super::*;
    #[test]
    fn parenthesis_optimization() {
        let mut tokens = load_test_data("/snippets/parsing/formulas/parenthesis-optimization.txt");
        let mut expr = Expression::parse(tokens.next().unwrap(), &mut tokens)
            .ok()
            .flatten()
            .unwrap();

        assert_eq!(expr.optimize_parentheses(), 2);
    }
}
