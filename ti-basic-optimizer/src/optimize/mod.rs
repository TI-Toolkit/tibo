mod expressions;

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
