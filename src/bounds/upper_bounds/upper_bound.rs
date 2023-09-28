use crate::problem_instance::{problem_instance::ProblemInstance, solution::Solution};



pub trait InitialUpperBound{
    /// Gets the upper bound on the given instance, given the current best
    fn get_upper_bound(&self, instance: &ProblemInstance) -> Solution;
}