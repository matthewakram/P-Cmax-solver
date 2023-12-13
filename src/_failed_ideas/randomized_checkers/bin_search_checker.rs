use rand::{seq::SliceRandom, thread_rng};

use crate::{
    common::timeout::Timeout,
    randomized_checkers::sss_ordered_randomized_checker::SSSOrderedRandomizedChecker,
};

use super::randomized_checker::RandomizedChecker;

#[derive(Clone)]
pub struct BinSearchChecker {}

impl RandomizedChecker for BinSearchChecker {
    fn is_sat(
        &self,
        part: &crate::problem_instance::partial_solution::PartialSolution,
        makespan_to_test: usize,
        timeout: &Timeout,
    ) -> Option<crate::problem_instance::solution::Solution> {
        if part.instance.num_processors <= 2 {
            return None;
        }

        let mut num_attempts: f64 = timeout.remaining_time();

        let mut lower = 0;
        let mut upper = part.instance.num_processors;

        while num_attempts > 0.0 && !timeout.time_finished() {
            let num_procs_to_fill = (lower + upper) / 2;
            let mut order: Vec<usize> = (0..part.instance.num_jobs).collect();
            order.shuffle(&mut thread_rng());
            let checker: SSSOrderedRandomizedChecker = SSSOrderedRandomizedChecker {
                job_order: order,
                num_procs_to_fill,
            };
            let checker_time = Timeout::new(timeout.remaining_time() / num_attempts);
            num_attempts -= 1.0;
            let sol = checker.is_sat(part, makespan_to_test, &checker_time);
            if sol.is_none() {
                // if it is none, then it is either unsat or timeout
                // if it is a timeout, then we need to fill up more processors
                if checker_time.time_finished() {
                    lower = num_procs_to_fill + 1;
                } else {
                    upper = num_procs_to_fill - 1;
                }
            } else {
                return sol;
            }
        }

        return None;
    }
}
