use titokens::{Token, Tokens};
use crate::parse::components::Component;
use crate::parse::Parse;

struct ExpressionParserState {
    in_list: bool,
    in_matrix: bool,
    in_matrix_row: bool,
}

pub struct Expression {
    pub components: Vec<Component>,
}

impl Parse for Expression {
    fn parse(token: Token, more: &mut Tokens) -> Option<Self> {
        todo!()
    }
}