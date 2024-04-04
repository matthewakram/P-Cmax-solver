use std::collections::{HashMap, HashSet};

use crate::{
     common, encoding::ilp_encoder::ILPEncoder,
    problem_instance::solution::Solution,
};
use bitvec::prelude::*;

#[derive(Clone)]
pub struct Naive {
    encoding: String,
    prec: bool,
}

impl Naive {
    pub fn new() -> Naive {
        return Naive {
            encoding: String::new(),
            prec: false,
        };
    }

    pub fn new_prec() -> Naive {
        return Naive {
            encoding: String::new(),
            prec: true,
        };
    }
}

impl ILPEncoder for Naive {
    fn encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        lower_bounds: usize,
        makespan: usize,
        _timeout: &crate::common::timeout::Timeout,
    ) -> bool {

        let mut formula = String::from("Minimize\nmakespan\nSubject To\n");

        let mut num_summands = vec![0;partial_solution.instance.num_processors];
        for proc in 0..partial_solution.instance.num_processors {
            let mut line = String::new();
            for job in 0..partial_solution.instance.num_jobs {
                if partial_solution.possible_allocations[job].contains(&proc) {
                    if num_summands[proc] == 0 {
                        line += &format!("{} a_{}_{} - makespan_{}_{} = 0\n", partial_solution.instance.job_sizes[job], job, proc, proc, num_summands[proc] + 1);
                    } else {
                        line += &format!("makespan_{}_{} + {} a_{}_{} - makespan_{}_{} = 0\n", proc, num_summands[proc], partial_solution.instance.job_sizes[job], job, proc, proc, num_summands[proc] + 1);
                    }
                    num_summands[proc] += 1;
                }
            }
            formula += &line;
        }

        for job in 0..partial_solution.instance.num_jobs{
            let mut line = String::new();
            for proc in &partial_solution.possible_allocations[job] {
                line += &format!("+ a_{}_{} ", job, proc);
            }
            line += "= 1\n";
            formula += &line;
        }

        for proc in 0..partial_solution.instance.num_processors {
            formula += &format!("makespan_{}_{} - makespan <= 0\n", proc, num_summands[proc]);
        }
        formula += &format!("makespan >= {}\n", lower_bounds);

        formula += "Binaries\n";

        for job in 0..partial_solution.instance.num_jobs {
            for proc in &partial_solution.possible_allocations[job] {
                formula += &format!(" a_{}_{}", job, proc);
            }
        }

        formula += "\nGenerals\nmakespan";

        formula+= "\nEnd\n";
        self.encoding = formula;
        println!("{}", self.encoding);
        return true;
    }

    fn get_encoding(&mut self) -> String {
        let mut out = String::new();
        std::mem::swap(&mut out, &mut self.encoding);
        return out;
    }

    fn decode(
        &self,
        instance: &crate::problem_instance::problem_instance::ProblemInstance,
        solution: HashMap<String, usize>,
    ) -> crate::problem_instance::solution::Solution {
        let mut assignment: Vec<usize> = vec![0; instance.num_jobs];
        let mut unassigned_jobs: HashSet<usize> = (0..instance.num_jobs).collect();

        let calculated_makespan = *solution.get("makespan").unwrap();
        for proc in 0..instance.num_processors {
            let mut current_node = 0;
            while current_node < calculated_makespan {
                let old_current = current_node;
                for job in 0..instance.num_jobs {
                    if !unassigned_jobs.contains(&job) {
                        continue;
                    }

                    let assigned = solution.get(&format!("v_{}_{}", current_node, job));
                    if assigned.is_some() {
                        let assigned = *assigned.unwrap();
                        if assigned == 1 {
                            unassigned_jobs.remove(&job);
                            assignment[job] = proc;
                            current_node += instance.job_sizes[job];
                            break;
                        }
                    }
                }
                if current_node == old_current {
                    break;
                }
            }
        }

        let makespan_check = common::common::calc_makespan(instance, &assignment);
        assert_eq!(makespan_check, calculated_makespan);

        return Solution {
            makespan: makespan_check,
            assignment,
        };
    }
}
