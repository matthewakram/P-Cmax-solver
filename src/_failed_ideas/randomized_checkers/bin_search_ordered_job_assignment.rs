use crate::common::timeout::Timeout;

use super::{
    ordered_job_assignment_checker::OrderedJobAssignmentChecker,
    randomized_checker::RandomizedChecker,
};

#[derive(Clone)]
pub struct BinSearchOrderedJobAssignmentChecker {}

impl RandomizedChecker for BinSearchOrderedJobAssignmentChecker {
    fn is_sat(
        &self,
        part: &crate::problem_instance::partial_solution::PartialSolution,
        makespan_to_test: usize,
        timeout: &Timeout,
    ) -> Option<crate::problem_instance::solution::Solution> {
        let mut num_attempts: f64 = timeout.remaining_time() * 0.6;

        let mut lower = 0;
        let mut upper = part.instance.num_jobs;

        while num_attempts > 0.0 && !timeout.time_finished() {
            let num_jobs_to_assign = (lower + upper) / 2;
            let order: Vec<usize> = (0..part.instance.num_jobs).collect();
            let checker = OrderedJobAssignmentChecker {
                job_order: order,
                num_jobs_to_assign,
            };
            let checker_time = Timeout::new(timeout.remaining_time() / num_attempts);
            num_attempts -= 1.0;
            let sol = checker.is_sat(part, makespan_to_test, &checker_time);
            if sol.is_none() {
                // if it is none, then it is either unsat or timeout
                // if it is a timeout, then we need to fill up more processors
                if checker_time.time_finished() {
                    lower = num_jobs_to_assign + 1;
                } else {
                    upper = num_jobs_to_assign - 1;
                }
            } else {
                return sol;
            }
        }

        return None;
    }
}
