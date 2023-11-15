use crate::{bounds::bound::Bound, problem_instance::solution::Solution, common::timeout::Timeout};



pub struct MiddleJobs{
}

impl Bound for MiddleJobs {
    fn bound(&self, problem: &crate::problem_instance::problem_instance::ProblemInstance, lower_bound: usize, upper_bound: Option<Solution>, _timeout: &Timeout) -> (usize, Option<Solution>) {
        if problem.num_processors + 1 >= problem.num_jobs {
            return (lower_bound, upper_bound);
        }
        let new_lower_bound = problem.job_sizes[problem.num_processors - 1] + problem.job_sizes[problem.num_processors];
        return (new_lower_bound.max(lower_bound), upper_bound)
    }
}