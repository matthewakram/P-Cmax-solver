use crate::{problem_instance::{partial_solution::PartialSolution, solution::Solution}, common::timeout::Timeout};


pub trait RandomizedChecker {
    fn is_sat(&self, part: &PartialSolution, makespan_to_test: usize, timeout: &Timeout) -> Option<Solution>;
}