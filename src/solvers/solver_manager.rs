use std::collections::HashMap;

use dyn_clone::DynClone;

use crate::{
    common::timeout::Timeout,
    problem_instance::{problem_instance::ProblemInstance, solution::Solution},
};

pub trait SolverManager: DynClone + Send {
    fn get_stats(&self) -> HashMap<String, f64> {
        return HashMap::new();
    }

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
