use crate::{bounds::bound::Bound, common::timeout::Timeout, problem_instance::solution::Solution};

pub struct MaxJobSize {}

impl Bound for MaxJobSize {
    fn bound(
        &self,
        problem: &crate::problem_instance::problem_instance::ProblemInstance,
        lower_bound: usize,
        upper_bound: Option<Solution>,
        _timeout: &Timeout,
    ) -> (usize, Option<Solution>) {
        let new_lower_bound = problem.job_sizes[0];
        return (new_lower_bound.max(lower_bound), upper_bound);
    }
}
