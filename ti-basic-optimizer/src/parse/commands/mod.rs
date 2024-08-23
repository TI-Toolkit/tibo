mod control_flow;
mod delvar_chain;
mod generic;
mod prgm;
mod setupeditor;

pub use control_flow::ControlFlow;
pub use delvar_chain::DelVarChain;
pub use generic::Generic;
use std::iter::once;

use crate::error_reporting::{expect_some, next_or_err, LineReport};
use crate::parse::commands::prgm::ProgramName;
use crate::parse::commands::setupeditor::SetUpEditor;
use crate::parse::components::StoreTarget;
use crate::parse::{expression::Expression, Parse, Reconstruct};
use titokens::{Token, Tokens, Version};

#[derive(Clone, Debug)]
pub enum Command {
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
    fn parse(token: Token, more: &mut Tokens) -> Result<Option<Self>, LineReport> {
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
    fn reconstruct(&self, version: &Version) -> Vec<Token> {
        match self {
            Command::ControlFlow(x) => x.reconstruct(version),
            Command::Generic(x) => x.reconstruct(version),
            Command::DelVarChain(x) => x.reconstruct(version),
            Command::SetUpEditor(x) => x.reconstruct(version),
            Command::Expression(x) => x.reconstruct(version),
            Command::ProgramInvocation(x) => x.reconstruct(version),
            Command::Store(x, target) => x
                .reconstruct(version)
                .into_iter()
                .chain(once(Token::OneByte(0x04)))
                .chain(target.reconstruct(version))
                .collect(),
        }
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
