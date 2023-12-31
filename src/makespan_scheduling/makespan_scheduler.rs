use dyn_clone::DynClone;

use crate::problem_instance::{problem_instance::ProblemInstance, solution::Solution};

pub trait MakespanScheduler: DynClone + Send {
    fn next_makespan(
        &mut self,
        instance: &ProblemInstance,
        solution: &Solution,
        lower: usize,
        upper: usize,
    ) -> usize;
}

dyn_clone::clone_trait_object!(MakespanScheduler);