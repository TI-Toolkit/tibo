use titokens::{Token, Tokens};
use crate::parse::Parse;

use crate::parse::components::*;

#[derive(Debug, Clone)]
pub enum Expression {
    Operator(Operator),
    Operand(Operand),
}

pub struct Builder<'a> {
    operand_stack: Vec<Expression>,
    operator_stack: Vec<Token>,

    paren_depth: u64,
    implicit_mul_allowed: bool,

    tokens: &'a mut Tokens,
}

impl<'a> Builder<'a> {
    #[must_use]
    pub fn new(tokens: &'a mut Tokens) -> Self {
        Self {
            operand_stack: vec![],
            operator_stack: vec![],

            paren_depth: 0,
            implicit_mul_allowed: false,

            tokens,
        }
    }

    #[must_use]
    pub fn build(mut self) -> Expression {
        while let Some(next) = self.tokens.next() {
            if !self.process_next(next) {
                break
            }
        }

        self.tokens.backtrack_once();

        self.finalize()
    }

    fn process_next(&mut self, next: Token) -> bool{
        if !self.process_operand_stack(next) {
            match next {
                Token::OneByte(0x10) => { // (
                    self.open_paren();
                    true
                }

                Token::OneByte(0x11) if self.paren_depth > 0 => { // )
                    self.close_paren();
                    true
                }

                Token::OneByte(0xB0) => {
                    self.operator_stack.push(next);

                    true
                }

                _ => if BinOp::recognize(next) {
                    self.push_binop(next);

                    true
                } else if UnOp::recognize(next) {
                    self.process_operator(next);

                    true
                } else {
                    false
                }
            }
        } else {
            true
        }
    }

    fn process_operand_stack(&mut self, next: Token) -> bool {
        if let Some(operand) = Operand::parse(next, self.tokens) {
            self.check_implicit_mul();

            self.emit_operand(operand.clone());

            if let Some(Token::OneByte(0x10)) = self.tokens.peek() { // (
                match &operand {
                    Operand::ListName(_) | Operand::Ans | Operand::MatrixName(_) => unimplemented!(),
                    _ => {}
                }
            }

            true
        } else if let Some(func) = FunctionCall::parse(next, self.tokens) {
            self.check_implicit_mul();
            self.operand_stack.push(Expression::Operator(Operator::FunctionCall(func)));
            self.implicit_mul_allowed = true;

            true
        } else {
            false
        }
    }

    fn emit_operand(&mut self, operand: Operand) {
        self.operand_stack.push(Expression::Operand(operand));
        self.implicit_mul_allowed = true;
    }

    fn open_paren(&mut self) {
        self.paren_depth += 1;
        self.operator_stack.push(Token::OneByte(0x10)); // (
        self.check_implicit_mul();
    }

    fn close_paren(&mut self) {
        self.paren_depth -= 1;

        while let Some(&token) = self.operator_stack.last() {
            if !self.process_operator(token) {
                break;
            } else {
                self.operator_stack.pop();
            }
        }

        if matches!(self.operator_stack.last(), Some(Token::OneByte(0x10))) { // (
            self.operator_stack.pop();

            self.implicit_mul_allowed = true;
        } else {
            panic!("Closing parenthesis assertion failed, please report this.")
        }
    }

    fn check_implicit_mul(&mut self) {
        if self.implicit_mul_allowed {
            self.push_binop(Token::OneByte(0x82)); // *
        }
    }

    fn push_binop(&mut self, operator: Token) {
        assert!(BinOp::recognize(operator));

        let precedence = BinOp::recognize_precedence(operator).unwrap();

        self.implicit_mul_allowed = false;

        while self.operator_stack.last().is_some_and(|tok|
            UnOp::recognize(*tok) ||
                (BinOp::recognize_precedence(*tok).unwrap_or(0) > precedence) // always false if not BinOp
        ) {
            let token = self.operator_stack.pop().unwrap();

            self.process_operator(token);
        }

        self.operator_stack.push(operator);
    }

    fn process_operator(&mut self, operator: Token) -> bool {
        if UnOp::recognize(operator) {
            let child = self.operand_stack.pop().unwrap();

            self.operand_stack.push(Expression::Operator(Operator::Unary(UnOp {
                kind: operator,
                child: Box::new(child),
            })));

            self.implicit_mul_allowed = false;

            true
        } else if BinOp::recognize(operator) {
            let right = self.operand_stack.pop().unwrap();
            let left = self.operand_stack.pop().unwrap();

            self.operand_stack.push(Expression::Operator(Operator::Binary(BinOp {
                kind: operator,
                right: Box::new(right),
                left: Box::new(left),
            })));

            self.implicit_mul_allowed = false;

            true
        } else {
            false
        }
    }

    fn valid(&self) -> bool {
        self.operator_stack.is_empty() && self.operand_stack.len() == 1
    }

    fn has_ambiguity(&self) -> bool {
        todo!();
    }

    fn finalize(&mut self) -> Expression {
        while let Some(x) = self.operator_stack.pop() {
            if !matches!(x, Token::OneByte(0x10)) { // (
                self.process_operator(x);
            }
        }

        assert!(self.valid());

        self.operand_stack[0].clone()
    }
}

impl Parse for Expression {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        let mut builder = Builder::new(more);
        builder.process_next(token);

        Some(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::load_test_data;

    #[test]
    fn quadratic() {
        let mut tokens = load_test_data("/snippets/parsing/formulas/quadratic.txt");

        let builder = Builder::new(&mut tokens);
        let _ = builder.build();
    }

    #[test]
    fn unop() {
        let mut tokens = load_test_data("/snippets/parsing/formulas/unop.txt");

        let builder = Builder::new(&mut tokens);
        let _ = builder.build();
    }

    #[test]
    fn manual_sum() {
        let mut tokens = load_test_data("/snippets/parsing/formulas/manual-sum.txt");

        let builder = Builder::new(&mut tokens);
        let _ = builder.build();
    }

    #[test]
    fn function_closing() {
        let mut tokens = load_test_data("/snippets/parsing/function-parsing/function-closing.txt");

        let builder = Builder::new(&mut tokens);
        let expr = builder.build();

        assert!(matches!(expr, Expression::Operator(Operator::Binary(_))));
    }
}