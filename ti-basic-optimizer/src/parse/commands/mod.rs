mod control_flow;
mod delvar_chain;
mod generic;

pub use control_flow::ControlFlow;
pub use delvar_chain::DelVarChain;
pub use generic::Generic;

use crate::parse::Parse;
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub enum Command {
    ControlFlow(ControlFlow),
    Generic(Generic),
    DelVarChain(DelVarChain),
}

impl Parse for Command {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        (Generic::parse(token, more).map(Command::Generic))
            .or_else(|| ControlFlow::parse(token, more).map(Command::ControlFlow))
            .or_else(|| DelVarChain::parse(token, more).map(Command::DelVarChain))
    }
}
