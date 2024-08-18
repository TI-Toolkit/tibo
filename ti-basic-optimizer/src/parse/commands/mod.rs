mod control_flow;
mod delvar_chain;
mod generic;

pub use control_flow::ControlFlow;
pub use delvar_chain::DelVarChain;
pub use generic::Generic;

use crate::error_reporting::{expect_some, next_or_err, LineReport};
use crate::parse::components::StoreTarget;
use crate::parse::{expression::Expression, Parse};
use titokens::{Token, Tokens};

#[derive(Clone, Debug)]
pub enum Command {
    ControlFlow(ControlFlow),
    Generic(Generic),
    DelVarChain(DelVarChain),
    Expression(Expression),
    Store(Expression, StoreTarget),
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
                    .map_err(|x| {
                        x.with_label(arrow_pos, "Store arrow is here.")
                    })?,
                )))
            } else {
                Ok(Some(Command::Expression(expr)))
            }
        } else {
            Ok(None)
        }
    }
}
