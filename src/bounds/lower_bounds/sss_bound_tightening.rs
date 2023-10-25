use bitvec::vec::BitVec;
use bitvec::prelude::*;

use crate::bounds::bound::Bound;


pub struct SSSBoundStrengthening {
}


impl Bound for SSSBoundStrengthening{
    fn bound(&self, problem: &crate::problem_instance::problem_instance::ProblemInstance, lower_bound: usize, upper_bound: Option<crate::problem_instance::solution::Solution>) -> (usize, Option<crate::problem_instance::solution::Solution>) {
        let upper = upper_bound.as_ref().unwrap().makespan;
        let mut reachable: BitVec = bitvec![0;upper+1];
        reachable.set(0, true);
        
        for job in 0..problem.num_jobs {
            for i in (0..upper).rev() {
                if reachable[i] && i + problem.job_sizes[job] <= upper {
                    reachable.set(i + problem.job_sizes[job], true);
                }
            }
        }
        
        for i in lower_bound..upper+1 {
            if reachable[i] {
                return (i, upper_bound);
            }
        }
        panic!("something went horribly wrong, there has to be a reachable value");
    }
}