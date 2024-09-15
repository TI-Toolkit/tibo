pub mod control_flow;
mod delvar_chain;
mod generic;
mod prgm;
mod setupeditor;

pub use control_flow::{ControlFlow, LabelName};
pub use delvar_chain::DelVarChain;
pub use prgm::ProgramName;
pub use setupeditor::SetUpEditor;

pub use generic::Generic;
use std::iter::once;

use crate::error_reporting::{expect_some, next_or_err, TokenReport};
use crate::parse::components::StoreTarget;
use crate::parse::{expression::Expression, Parse, Reconstruct};
use crate::Config;
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub enum Command {
    None,
    ControlFlow(ControlFlow),
    Generic(Generic),
    DelVarChain(DelVarChain),
    SetUpEditor(SetUpEditor),
    Expression(Expression),
    Store(Expression, StoreTarget),
    ProgramInvocation(ProgramName),
}

impl Parse for Command {
    #[allow(unused_parens)]
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, TokenReport> {
        if let Some(cmd) = Generic::parse(token, more)?.map(Command::Generic) {
            Ok(Some(cmd))
        } else if let Some(cmd) = ControlFlow::parse(token, more)?.map(Command::ControlFlow) {
            Ok(Some(cmd))
        } else if let Some(cmd) = DelVarChain::parse(token, more)?.map(Command::DelVarChain) {
            Ok(Some(cmd))
        } else if let Some(cmd) = ProgramName::parse(token, more)?.map(Command::ProgramInvocation) {
            Ok(Some(cmd))
        } else if let Some(cmd) = SetUpEditor::parse(token, more)?.map(Self::SetUpEditor) {
            Ok(Some(cmd))
        } else if let Some(expr) = Expression::parse(token, more)? {
            if more.peek() == Some(Token::OneByte(0x04)) {
                let arrow_pos = more.current_position();
                more.next();

                Ok(Some(Command::Store(
                    expr,
                    expect_some!(
                        StoreTarget::parse(next_or_err!(more)?, more)?,
                        more,
                        1,
                        "a store target",
                        "Parsing failed here."
                    )
                    .map_err(|x| x.with_label(arrow_pos, "Store arrow is here."))?,
                )))
            } else {
                Ok(Some(Command::Expression(expr)))
            }
        } else {
            Ok(None)
        }
    }
}

impl Reconstruct for Command {
    fn reconstruct(&self, config: &Config) -> Vec<Token> {
        let mut line = match self {
            Command::ControlFlow(x) => x.reconstruct(config),
            Command::Generic(x) => x.reconstruct(config),
            Command::DelVarChain(x) => x.reconstruct(config),
            Command::SetUpEditor(x) => x.reconstruct(config),
            Command::Expression(x) => x.reconstruct(config),
            Command::ProgramInvocation(x) => x.reconstruct(config),
            Command::Store(x, target) => {
                let mut expr = x.reconstruct(config);
                Expression::strip_closing_parenthesis(&mut expr);
                expr.into_iter()
                    .chain(once(Token::OneByte(0x04)))
                    .chain(target.reconstruct(config))
                    .collect()
            }
            Command::None => return vec![],
        };

        Expression::strip_closing_parenthesis(&mut line);

        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_files::load_test_data;

    #[test]
    fn store() {
        let mut tokens = load_test_data("/snippets/parsing/commands/store.txt");

        let cmd = Command::parse(tokens.next().unwrap(), &mut tokens)
            .unwrap()
            .unwrap();
        assert!(matches!(cmd, Command::Store(_, _)));
    }
}
