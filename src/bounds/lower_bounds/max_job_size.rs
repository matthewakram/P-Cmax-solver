use crate::{bounds::bound::Bound, problem_instance::solution::Solution};


pub struct MaxJobSize{
}


impl Bound for MaxJobSize {
    fn bound(&self, problem: &crate::problem_instance::problem_instance::ProblemInstance, lower_bound: usize, upper_bound: Option<Solution>, timeout: f64) -> (usize, Option<Solution>) {
        //TODO: we can assume that this is the first processor
        let new_lower_bound = problem.job_sizes[0];
        return (new_lower_bound.max(lower_bound), upper_bound)
    }
}