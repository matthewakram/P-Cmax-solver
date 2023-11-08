use std::time::Instant;

use crate::randomized_checkers::sss_ordered_randomized_checker::SSSOrderedRandomizedChecker;

use super::randomized_checker::RandomizedChecker;


pub struct DescendingMultiSSSRandomizedChecker{

}

impl RandomizedChecker for DescendingMultiSSSRandomizedChecker {
    fn is_sat(&self, part: &crate::problem_instance::partial_solution::PartialSolution, makespan_to_test: usize, timeout: f64) -> Option<crate::problem_instance::solution::Solution> {
        let order: Vec<usize> = (0..part.instance.num_jobs).into_iter().collect();

        if part.instance.num_processors <= 3 {
            return None;
        }

        let mut timeout = timeout;

        for num_procs_to_fill in (1..part.instance.num_processors-2).rev() {
            let start_time = Instant::now();
            let checker = SSSOrderedRandomizedChecker{ job_order: order.clone(), num_procs_to_fill, text_file_to_use: "./randomized_checking_encoding)".to_string() };

            let sol = checker.is_sat(part, makespan_to_test, timeout);
            if sol.is_some() {
                return sol;
            }
            timeout -=  start_time.elapsed().as_secs_f64();
        }
        return None;
    }
}