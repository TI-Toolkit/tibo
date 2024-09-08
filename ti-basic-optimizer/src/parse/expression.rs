use crate::error_reporting::TokenReport;
use crate::parse::components::*;
use crate::parse::{Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

#[derive(Debug, Clone)]
pub enum Expression {
    Operator(Operator),
    Operand(Operand),
}

struct Builder<'a> {
    operand_stack: Vec<Expression>,
    operator_stack: Vec<Token>,

    paren_depth: u64,
    implicit_mul_allowed: bool,

    expression_start: usize,
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

            expression_start: tokens.current_position(),
            tokens,
        }
    }

    pub fn build(mut self) -> Result<Option<Expression>, TokenReport> {
        while let Some(next) = self.tokens.next() {
            if !self.process_next(next)? {
                break;
            }
        }

        self.tokens.backtrack_once();

        self.finalize()
    }

    fn error(&self, code: usize) -> TokenReport {
        TokenReport::new(
            self.tokens.current_position(),
            "Expression parsing error",
            Some("Please report this if this is unexpected."),
        )
        .with_span_label(
            self.expression_start..self.tokens.current_position(),
            "This may be incorrect",
        )
        .with_label(
            self.tokens.current_position(),
            "This token triggered the error.",
        )
        .with_code(code)
    }

    #[allow(clippy::let_and_return)]
    fn process_next(&mut self, next: Token) -> Result<bool, TokenReport> {
        let result = if !self.process_operand_stack(next)? {
            match next {
                Token::OneByte(0x10) => {
                    // (
                    self.open_paren()?;
                    Ok(true)
                }

                Token::OneByte(0x11) if self.paren_depth > 0 => {
                    // )
                    self.close_paren()?;
                    Ok(true)
                }

                Token::OneByte(0xB0) => {
                    // ~
                    self.operator_stack.push(next);
                    self.implicit_mul_allowed = false;

                    Ok(true)
                }

                _ => {
                    if BinOp::recognize(next) {
                        self.push_binop(next)?;

                        Ok(true)
                    } else if UnOp::recognize(next) {
                        self.process_operator(next)?;

                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
            }
        } else {
            Ok(true)
        };

        // dbg!(next, self.implicit_mul_allowed, &self.operator_stack, &self.operand_stack);

        result
    }

    fn process_operand_stack(&mut self, next: Token) -> Result<bool, TokenReport> {
        if let Some(operand) = Operand::parse(next, self.tokens)? {
            self.check_implicit_mul()?;

            self.emit_operand(operand.clone());

            if let Some(Token::OneByte(0x10)) = self.tokens.peek() {
                // (
                match &operand {
                    Operand::Ans => {
                        unimplemented!()
                    }
                    _ => {}
                }
            }

            self.implicit_mul_allowed = true;

            Ok(true)
        } else if let Some(func) = FunctionCall::parse(next, self.tokens)? {
            self.check_implicit_mul()?;
            self.operand_stack
                .push(Expression::Operator(Operator::FunctionCall(func)));
            self.implicit_mul_allowed = true;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn emit_operand(&mut self, operand: Operand) {
        self.operand_stack.push(Expression::Operand(operand));
        self.implicit_mul_allowed = true;
    }

    fn open_paren(&mut self) -> Result<(), TokenReport> {
        self.paren_depth += 1;
        self.check_implicit_mul()?;
        self.operator_stack.push(Token::OneByte(0x10)); // (

        Ok(())
    }

    fn close_paren(&mut self) -> Result<(), TokenReport> {
        self.paren_depth -= 1;

        while let Some(&token) = self.operator_stack.last() {
            if !self.process_operator(token)? {
                break;
            } else {
                self.operator_stack.pop();
            }
        }

        if matches!(self.operator_stack.last(), Some(Token::OneByte(0x10))) {
            // (
            self.operator_stack.pop();

            self.implicit_mul_allowed = true;
        } else {
            Err(self.error(1))?;
        }

        Ok(())
    }

    fn check_implicit_mul(&mut self) -> Result<(), TokenReport> {
        if self.implicit_mul_allowed {
            self.push_binop(Token::OneByte(0x82))?; // *
        }

        Ok(())
    }

    fn push_binop(&mut self, operator: Token) -> Result<(), TokenReport> {
        assert!(BinOp::recognize(operator));

        let precedence = BinOp::recognize_precedence(operator).unwrap();

        self.implicit_mul_allowed = false;

        while self.operator_stack.last().is_some_and(
            |tok| {
                UnOp::recognize(*tok)
                    || (BinOp::recognize_precedence(*tok).unwrap_or(0) >= precedence)
            }, // always false if not BinOp
        ) {
            let token = self.operator_stack.pop().unwrap();

            self.process_operator(token)?;
        }

        self.operator_stack.push(operator);

        Ok(())
    }

    fn process_operator(&mut self, operator: Token) -> Result<bool, TokenReport> {
        if UnOp::recognize(operator) {
            let child = self.operand_stack.pop().ok_or_else(|| self.error(5))?;

            self.operand_stack
                .push(Expression::Operator(Operator::Unary(UnOp {
                    kind: operator,
                    child: Box::new(child),
                })));

            if operator != Token::OneByte(0xB0) {
                self.implicit_mul_allowed = true
            }

            Ok(true)
        } else if BinOp::recognize(operator) {
            let right = self.operand_stack.pop().ok_or_else(|| self.error(2))?;
            let left = self.operand_stack.pop().ok_or_else(|| self.error(3))?;

            self.operand_stack
                .push(Expression::Operator(Operator::Binary(BinOp {
                    kind: operator,
                    right: Box::new(right),
                    left: Box::new(left),
                })));

            self.implicit_mul_allowed = false;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn valid(&self) -> bool {
        self.operator_stack.is_empty() && self.operand_stack.len() == 1
    }

    fn has_ambiguity(&self) -> bool {
        todo!();
    }

    fn finalize(&mut self) -> Result<Option<Expression>, TokenReport> {
        while let Some(x) = self.operator_stack.pop() {
            if !matches!(x, Token::OneByte(0x10)) {
                // everything besides (
                self.process_operator(x)?;
            }
        }

        if !self.valid() {
            Err(self.error(4))?;
        }

        assert!(self.valid());

        Ok(self.operand_stack.first().cloned())
    }
}

impl Parse for Expression {
    fn parse(_token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        more.backtrack_once();
        let builder = Builder::new(more);

        builder.build()
    }
}

impl Reconstruct for Expression {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        match self {
            Expression::Operator(operator) => operator.reconstruct(config),
            Expression::Operand(operand) => operand.reconstruct(config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::{load_test_data, test_version};
    use titokens::{version, Tokenizer};

    macro_rules! test_case {
        ($name: ident, $path: expr, $version: expr) => {
            #[test]
            fn $name() {
                let data = load_test_data($path);
                let mut tokens = data.clone();

                let builder = Builder::new(&mut tokens);
                let parsed = builder.build().unwrap().unwrap();

                // dbg!(Tokenizer::new($version.clone(), "en").stringify(&parsed.reconstruct($version)));
                assert_eq!(
                    parsed.reconstruct(&$version.into()),
                    data.collect::<Vec<_>>()
                );
            }
        };

        ($name: ident, $path: expr) => {
            test_case!($name, $path, test_files::test_version());
        };
    }

    test_case!(quadratic, "/snippets/parsing/formulas/quadratic.txt");
    test_case!(unop, "/snippets/parsing/formulas/unop.txt");
    test_case!(
        manual_sum,
        "/snippets/parsing/formulas/manual-sum.txt",
        version::LATEST_MONO.clone()
    );

    #[test]
    fn function_closing() {
        let mut tokens = load_test_data("/snippets/parsing/function-parsing/function-closing.txt");

        let builder = Builder::new(&mut tokens);
        let expr = builder.build().ok().unwrap();

        assert!(matches!(
            expr,
            Some(Expression::Operator(Operator::Binary(_)))
        ));
    }

    test_case!(
        exp_assoc1,
        "/snippets/parsing/associativity/exponentiation.txt"
    );
    test_case!(
        exp_assoc2,
        "/snippets/parsing/associativity/exponentiation2.txt"
    );

    test_case!(
        scrabble_score,
        "/snippets/parsing/formulas/scrabble-score.txt"
    );

    test_case!(
        iverson_bracket,
        "/snippets/parsing/formulas/iverson-bracket.txt"
    );
}
