use crate::data::Program;

mod closing_parenthesis_elimination;
mod closing_parenthesis_for_loop;

trait OptimizationPass {
    fn optimize(program: &mut Program);
}