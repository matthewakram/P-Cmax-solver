
use crate::problem_instance::partial_solution::PartialSolution;

use super::simplification_rule::SimpRule;

pub struct FillUpRule {}

// the half size rule states that for all equivalent processors, the elements that weigh more than half of the remaining makespan
// are mututally exclusive on each machine and can thus be ordered linearly
impl SimpRule for FillUpRule {
    fn simplify(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        max_makespan: usize,
    ) -> PartialSolution {
        let mut out = partial_solution.clone();


        for process in 0..out.instance.num_processors {
            for job in 0..out.instance.num_jobs {
                if out.instance.job_sizes[job] == max_makespan - out.assigned_makespan[process] && out.possible_allocations[job].contains(&process) {
                    out.possible_allocations[job] = vec![process];
                    out.assigned_makespan[process] = max_makespan;
                    break;
                }
            }
        }
        return out;
    }
}
