use crate::{problem_instance::{partial_solution::PartialSolution, solution::Solution}, solvers::{solver::SatSolver, sat_solver::kissat::Kissat}, encoding::{pb_bdd_pysat::PbPysatEncoder, encoder::Encoder, basic_with_precedence::Precedence}, common::timeout::Timeout};

use super::randomized_checker::RandomizedChecker;

#[derive(Clone)]
pub struct OrderedJobAssignmentChecker {
    pub job_order: Vec<usize>,
    pub num_jobs_to_assign: usize,
}


impl RandomizedChecker for OrderedJobAssignmentChecker {
    fn is_sat(&self, part: &crate::problem_instance::partial_solution::PartialSolution, makespan_to_test: usize, timeout: &Timeout) -> Option<Solution> {
        let mut reduced_sol = PartialSolution {
             instance: part.instance.clone(), 
             possible_allocations: vec![(0..part.instance.num_processors).into_iter().collect(); part.instance.num_jobs],
              assigned_makespan: vec![0;part.instance.num_processors] 
            };
        
        for i in 0..self.num_jobs_to_assign {
            let too_far = !reduced_sol.assigned_makespan.iter().all(|x| *x <= makespan_to_test);
            if too_far {
                return None;
            }
            if reduced_sol.possible_allocations[self.job_order[i]].len() == 1 {
                continue;
            }
            let (min_proc, _) = reduced_sol.assigned_makespan.iter().enumerate().min_by_key(|(_,x)| *x).unwrap();

            reduced_sol.possible_allocations[self.job_order[i]] = vec![min_proc];
            reduced_sol.assigned_makespan[min_proc] += reduced_sol.instance.job_sizes[self.job_order[i]];
        }
        let too_far = !reduced_sol.assigned_makespan.iter().all(|x| *x <= makespan_to_test);
        if too_far {
            return None;
        }

        if (&reduced_sol.possible_allocations).into_iter().all(|x: &Vec<usize>| x.len() == 1) {
            let sol = Solution {
                makespan: *(reduced_sol.assigned_makespan.iter().max().unwrap()),
                assignment: reduced_sol.possible_allocations.into_iter().map(|x: Vec<usize>| x[0]).collect(),
            };
            if sol.makespan > makespan_to_test {
                return None;
            }
        
            return Some(sol);
        }

        let mut encoder = Box::new( Precedence::new(Box::new(PbPysatEncoder::new()), 1));
        encoder.basic_encode(&reduced_sol, makespan_to_test, timeout);
        let encoding = encoder.output();
        let solver = Kissat{};
        let solution = solver.solve(&encoding, encoder.get_num_vars(),  timeout);

        
        if solution.is_sat() {
            let sol = solution.unwrap();
            return  Some(encoder.decode(&reduced_sol.instance, sol.as_ref().unwrap()));
        }
        return None;
    }
}