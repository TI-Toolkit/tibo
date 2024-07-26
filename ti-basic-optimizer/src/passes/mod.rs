use crate::data::Program;
use crate::Settings;

mod closing_parenthesis_elimination;
mod closing_parenthesis_for_loop;

trait OptimizationPass {
    fn optimize(program: &mut Program, settings: &Settings);
}