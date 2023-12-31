use dyn_clone::DynClone;

use crate::{problem_instance::{problem_instance::ProblemInstance, solution::Solution}, common::timeout::Timeout};


pub trait SolverManager: DynClone + Send {
    fn solve(
        &mut self,
        instance: &ProblemInstance,
        lower: usize,
        upper: &Solution,
        timeout: &Timeout,
        verbose: bool,
    ) -> Option<Solution>;
}

dyn_clone::clone_trait_object!(SolverManager);