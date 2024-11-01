use dyn_clone::DynClone;

use crate::{
    common::timeout::Timeout,
    problem_instance::{
        partial_solution::PartialSolution, problem_instance::ProblemInstance, solution::Solution,
    },
};

pub trait CPLEXModelEncoder: DynClone + Send {
    fn encode(
        &mut self,
        partial_solution: &PartialSolution,
        lower_bounds: usize,
        makespan: usize,
        timeout: &Timeout,
    ) -> bool;
    fn get_encoding(&self) -> String;
    fn get_mod_file_path(&self) -> String;
    fn decode(&self, instance: &ProblemInstance, solution: String) -> Solution;
}

dyn_clone::clone_trait_object!(CPLEXModelEncoder);
