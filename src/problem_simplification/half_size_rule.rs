use crate::problem_instance::partial_solution::{self, PartialSolution};

use super::simplification_rule::SimpRule;

pub struct HalfSizeRule {}

// the half size rule states that for all equivalent processors, the elements that weigh more than half of the remaining makespan
// are mututally exclusive on each machine and can thus be ordered linearly
impl SimpRule for HalfSizeRule {
    fn simplify(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        max_makespan: usize,
    ) -> Option<PartialSolution> {
        let mut out = partial_solution::PartialSolution::new(partial_solution.instance.clone());
        for job in 0..partial_solution.instance.num_processors {
            if partial_solution.instance.job_sizes[job] > max_makespan / 2 {
                out.possible_allocations[job] = vec![job];
                out.assigned_makespan[job] = out.instance.job_sizes[job];
                continue;
            }
            break;
        }
        return Some(out);
    }
}
