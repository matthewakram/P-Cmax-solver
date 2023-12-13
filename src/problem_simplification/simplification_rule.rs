use crate::problem_instance::partial_solution::PartialSolution;

pub trait SimpRule {
    fn simplify(
        &mut self,
        partial_solution: &PartialSolution,
        max_makespan: usize,
    ) -> Option<PartialSolution>;
}
