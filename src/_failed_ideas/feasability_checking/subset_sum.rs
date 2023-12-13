use super::feasability_checker::FeasabilityChecker;
use crate::bitvec::prelude::*;
use crate::bitvec::vec::BitVec;

pub struct SubsetSum {}

impl FeasabilityChecker for SubsetSum {
    fn get_next_feasable(
        &self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan_to_check: usize,
        lower_bound: usize,
    ) -> Option<usize> {
        let mut reachable: BitVec = bitvec![0;makespan_to_check];
        reachable.set(0, true);

        for job in 0..partial_solution.instance.num_jobs {
            for i in (0..makespan_to_check).rev() {
                if reachable[i] && i + partial_solution.instance.job_sizes[job] <= makespan_to_check
                {
                    if i + partial_solution.instance.job_sizes[job] == makespan_to_check {
                        return Some(makespan_to_check);
                    }
                    reachable.set(i + partial_solution.instance.job_sizes[job], true);
                }
            }
        }

        for i in (lower_bound..makespan_to_check).rev() {
            if reachable[i] {
                return Some(i);
            }
        }
        return None;
    }
}
