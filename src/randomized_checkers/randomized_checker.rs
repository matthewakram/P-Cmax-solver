use dyn_clone::DynClone;

use crate::{problem_instance::{partial_solution::PartialSolution, solution::Solution}, common::timeout::Timeout};


pub trait RandomizedChecker : DynClone + Send {
    fn is_sat(&self, part: &PartialSolution, makespan_to_test: usize, timeout: &Timeout) -> Option<Solution>;
}

dyn_clone::clone_trait_object!(RandomizedChecker);