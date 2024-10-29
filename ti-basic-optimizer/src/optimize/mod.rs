use crate::parse::Program;
use crate::Config;

mod control_flow;
mod expressions;
mod strategies;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum Priority {
    /// Provides a reasonable mix of both speed and size optimizations.
    #[default]
    Neutral,
    /// Disables optimizations which would slow the program down.
    Speed,
    /// Disables optimizations which would increase the program's size.
    Size,
}

impl Program {
    pub fn optimize(&mut self, _config: &Config) {
        self.optimize_label_names();

        for statement in self.lines.iter_mut() {
            statement.optimize_parentheses();
        }
    }
}
