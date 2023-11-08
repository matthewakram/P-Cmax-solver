use crate::{solvers::solver::SatResult, problem_instance::{partial_solution::PartialSolution, solution::Solution}};


pub trait RandomizedChecker {
    fn is_sat(&self, part: &PartialSolution, makespan_to_test: usize, timeout: f64) -> Option<Solution>;
}