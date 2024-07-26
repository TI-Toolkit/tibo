use titokens::Token;

use crate::data::Program;
use crate::Settings;

use super::OptimizationPass;

struct ClosingParenthesisForLoop {}

impl OptimizationPass for ClosingParenthesisForLoop {
    /// # Closing Parenthesis For Loop
    /// A `For(` loop immediately followed by either
    ///  - an `If ` without `Then`
    ///  - or `IS>(` or `DS<(`
    ///
    /// where the condition is falsy will generally run an order of magnitude slower if the closing
    /// parenthesis is removed. When `--size` is enabled, a parenthesis will always be removed in
    /// this situation. When `--speed` is enabled, the parenthesis will always be added in this
    /// situation.
    ///
    /// ## Special Considerations
    /// When two or more closing parenthesis are required to close the For loop, it is better to
    /// insert a newline between the `For(` statement and the conditional.
    fn optimize(program: &mut Program, settings: &Settings) {
        for line_index in 0..program.lines.len() {
            let line = &mut program.lines[line_index];

            if matches!(line.tokens[0], Token::OneByte(0xD3)) && // For(
                line_index != program.lines.len() - 1 &&
                matches!(program.lines[line_index + 1].tokens[0], Token::OneByte(0xCE | 0xDA | 0xDB)) {}
        }
    }
}