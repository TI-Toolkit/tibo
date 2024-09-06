use crate::Config;
use crate::parse::Program;

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
    pub fn optimize(&mut self, config: &Config) {
        for command in self.lines.iter_mut() {
            command.optimize_parentheses();
        }
    }
}