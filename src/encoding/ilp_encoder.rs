use std::collections::HashMap;

use dyn_clone::DynClone;

use crate::{
    common::timeout::Timeout,
    problem_instance::{
        partial_solution::PartialSolution, problem_instance::ProblemInstance, solution::Solution,
    },
};


pub trait ILPEncoder: DynClone + Send {
    fn encode(
        &mut self,
        partial_solution: &PartialSolution,
        lower_bounds: usize,
        makespan: usize,
        timeout: &Timeout,
    ) -> bool;
    fn get_encoding(&mut self) -> String;
    fn decode(&self, instance: &ProblemInstance, solution: HashMap<String, usize>) -> Solution;
}

dyn_clone::clone_trait_object!(ILPEncoder);