
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
    ) ->Option<PartialSolution> {
        let mut out: PartialSolution = partial_solution.clone();
        //println!("job sizes {:?}", partial_solution.instance.job_sizes);
        //println!("assigned {:?}", partial_solution.assigned_makespan);
        //for a in &partial_solution.possible_allocations {
        //    println!("poss all {:?}", a);
        //}


        for job in 0..partial_solution.instance.num_jobs {
            let mut finalized_possible_allocations_i = vec![];
            if partial_solution.possible_allocations[job].len() == 1 {
                continue;
            }
            for proc in &partial_solution.possible_allocations[job] {
                if partial_solution.instance.job_sizes[job] <= max_makespan - out.assigned_makespan[*proc] {
                    finalized_possible_allocations_i.push(*proc);
                } 
            }
            if finalized_possible_allocations_i.len() == 0 {
                return None;
            }
            out.possible_allocations[job] = finalized_possible_allocations_i;
            if out.possible_allocations[job].len() == 1{
                out.assigned_makespan[out.possible_allocations[job][0]] += out.instance.job_sizes[job];
            }
        }
        
        return Some(out);
    }
}
