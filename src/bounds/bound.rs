use crate::problem_instance::{problem_instance::ProblemInstance, solution::Solution};


pub trait Bound {
    fn bound(&self, problem: &ProblemInstance, lower_bound: usize, upper_bound: Option<Solution>) -> (usize, Option<Solution>);
}