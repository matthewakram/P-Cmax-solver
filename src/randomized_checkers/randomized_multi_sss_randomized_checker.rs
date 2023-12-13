use rand::{seq::SliceRandom, thread_rng};

use crate::{
    common::timeout::Timeout,
    randomized_checkers::sss_ordered_randomized_checker::SSSOrderedRandomizedChecker,
};

use super::randomized_checker::RandomizedChecker;

#[derive(Clone)]
pub struct RandomizedMultiSSSRandomizedChecker {}

impl RandomizedChecker for RandomizedMultiSSSRandomizedChecker {
    fn is_sat(
        &self,
        part: &crate::problem_instance::partial_solution::PartialSolution,
        makespan_to_test: usize,
        timeout: &Timeout,
    ) -> Option<crate::problem_instance::solution::Solution> {
        if part.instance.num_processors <= 2 {
            return None;
        }

        let mut num_attempts = (part.instance.num_processors - 3) as f64;

        for num_procs_to_fill in 1..(part.instance.num_processors - 1) {
            if part.instance.num_processors >= 12 && num_procs_to_fill % 2 == 0 {
                continue;
            }
            if part.instance.num_processors >= 25 && num_procs_to_fill % 3 == 0 {
                continue;
            }
            if part.instance.num_processors >= 100 && num_procs_to_fill % 5 == 0 {
                continue;
            }
            let mut order: Vec<usize> = (0..part.instance.num_jobs).collect();
            order.shuffle(&mut thread_rng());
            let checker: SSSOrderedRandomizedChecker = SSSOrderedRandomizedChecker {
                job_order: order.clone(),
                num_procs_to_fill,
            };

            let sol = checker.is_sat(
                part,
                makespan_to_test,
                &Timeout::new(timeout.remaining_time() / num_attempts),
            );
            num_attempts -= 1.0;
            if sol.is_some() {
                return sol;
            }
            if timeout.time_finished() {
                return None;
            }
        }
        return None;
    }
}
