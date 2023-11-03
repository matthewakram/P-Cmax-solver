
use crate::problem_instance::partial_solution::PartialSolution;

use super::simplification_rule::SimpRule;

pub struct FinalizeRule {}

// the half size rule states that for all equivalent processors, the elements that weigh more than half of the remaining makespan
// are mututally exclusive on each machine and can thus be ordered linearly
impl SimpRule for FinalizeRule {
    fn simplify(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        max_makespan: usize,
    ) -> PartialSolution {
        let mut out: PartialSolution = partial_solution.clone();


        for job in 0..partial_solution.instance.num_jobs {
            let mut finalized_possible_allocations_i = vec![];
            if partial_solution.possible_allocations[job].len() == 1 {
                continue;
            }
            for proc in &partial_solution.possible_allocations[job] {
                if partial_solution.instance.job_sizes[job] <= max_makespan - partial_solution.assigned_makespan[*proc] {
                    finalized_possible_allocations_i.push(*proc);
                } 
            }
            assert_ne!(finalized_possible_allocations_i.len(), 0);
            out.possible_allocations[job] = finalized_possible_allocations_i;
            if out.possible_allocations[job].len() == 1{
                out.assigned_makespan[out.possible_allocations[job][0]] += out.instance.job_sizes[job];
            }
        }
        return out;
    }
}
