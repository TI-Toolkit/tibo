use crate::data::Program;

use super::OptimizationPass;

struct ClosingParenthesisElimination {}

impl OptimizationPass for ClosingParenthesisElimination {
    /// # Closing Parenthesis Elimination
    /// Closing Parentheses can be omitted in several situations:
    /// `L1(A+1)->B` is
    fn optimize(program: &mut Program) {
        for line_index in 0..program.lines.len() {
            let line = &mut program.lines[line_index];
        }
    }
}