use crate::problem_instance::partial_solution::PartialSolution;


pub trait FeasabilityChecker {
    fn get_next_feasable(&self, partial_solution: &PartialSolution, makespan_to_check: usize, lower_bound: usize) -> Option<usize>;
}