use crate::{problem_instance::{partial_solution::PartialSolution, solution::Solution}, solvers::{solver::SatSolver, sat_solver::kissat::Kissat}, encoding::{pb_bdd_pysat::PbPysatEncoder, encoder::Encoder}, common::timeout::Timeout};

use super::randomized_checker::RandomizedChecker;

#[derive(Clone)]
pub struct SSSOrderedRandomizedChecker{
    pub job_order: Vec<usize>,
    pub num_procs_to_fill: usize,
}

impl SSSOrderedRandomizedChecker{
    fn fill_proc(&self, part: PartialSolution, makespan_to_test: usize, proc: usize) -> PartialSolution{
        assert!(part.instance.num_jobs < u16::MAX as usize);
        
        let mut reachable: Vec<u16> = vec![u16::MAX;makespan_to_test+1];

        
        let current_makespan = part.assigned_makespan[proc];

        
        reachable[current_makespan] = 0;
        for job in &self.job_order{
            let job = *job;
            if part.possible_allocations[job].len() !=1 && part.possible_allocations[job].contains(&proc){
                let job_size = part.instance.job_sizes[job];
                for i in (current_makespan + job_size..makespan_to_test+1).rev() {
                    if reachable[i] == u16::MAX && reachable[i-job_size] != u16::MAX {
                        reachable[i] = job as u16;
                        if i == makespan_to_test {
                            break;
                        }
                    }
                }
                if reachable[makespan_to_test] != u16::MAX {
                    break;
                }
            }
        }

        let mut pointer = makespan_to_test;
        while reachable[pointer] == u16::MAX{
            pointer -= 1;
        }
        assert!(pointer != 0);

        let mut out = part.clone();
        out.assigned_makespan[proc] = pointer;

        while pointer != current_makespan {
            let job_num = reachable[pointer] as usize;
            out.possible_allocations[job_num] = vec![proc];
            pointer -= part.instance.job_sizes[job_num];
        }

        for job in 0..out.instance.num_jobs {
            if out.possible_allocations[job].len() == 1 {
                continue;
            }
            let index_of_currently_filled_proc = out.possible_allocations[job].iter().enumerate().find(|(_,x)| **x == proc);
            if index_of_currently_filled_proc.is_none() {
                continue;
            }
            let index = index_of_currently_filled_proc.unwrap().0;
            out.possible_allocations[job].remove(index);
            if out.possible_allocations[job].len() == 1 {
                let proc = out.possible_allocations[job][0];
                out.assigned_makespan[proc] += out.instance.job_sizes[job];
            }
        }
        return out;
    }
}

impl RandomizedChecker for SSSOrderedRandomizedChecker {
    fn is_sat(&self, part: &crate::problem_instance::partial_solution::PartialSolution, makespan_to_test: usize, timeout: &Timeout) -> Option<Solution> {
        let mut reduced_sol = part.clone();
        
        let proc_to_fill = 0;
        //while part.assigned_makespan[proc_to_fill] == makespan_to_test {
        //    proc_to_fill +=1;
        //}
        
        for i in 0..self.num_procs_to_fill {
            if proc_to_fill + i >= part.instance.num_processors {
                break;
            }

            let too_far = !reduced_sol.assigned_makespan.iter().all(|x| *x <= makespan_to_test);
            if too_far {
                return None;
            }
            reduced_sol = self.fill_proc(reduced_sol, makespan_to_test, proc_to_fill + i);
        }
        let too_far = !reduced_sol.assigned_makespan.iter().all(|x| *x <= makespan_to_test);
        if too_far {
            return None;
        }


        

        if (&reduced_sol.possible_allocations).into_iter().all(|x| x.len() == 1) {
            let sol = Solution {
                makespan: *(reduced_sol.assigned_makespan.iter().max().unwrap()),
                assignment: reduced_sol.possible_allocations.into_iter().map(|x: Vec<usize>| x[0]).collect(),
            };
            if sol.makespan > makespan_to_test {
                return None;
            }
        
            return Some(sol);
        }

        let mut encoder = Box::new(PbPysatEncoder::new());
        let success = encoder.basic_encode(&reduced_sol, makespan_to_test, timeout, 500_000_000);
        if !success {
            return None;
        }
        let encoding = encoder.output();
        let mut solver = Kissat::new();
        let solution = solver.solve(encoding, encoder.get_num_vars(),  timeout);

        
        if solution.is_sat() {
            let sol = solution.unwrap();
            return  Some(encoder.decode(&reduced_sol.instance, sol.as_ref().unwrap()));
        }
        return None;
    }
}