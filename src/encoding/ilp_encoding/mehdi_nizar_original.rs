use std::collections::{HashMap, HashSet};

use crate::{common, encoding::ilp_encoder::ILPEncoder, problem_instance::solution::Solution};
use bitvec::prelude::*;

#[derive(Clone)]
pub struct MehdiNizarOriginalEncoder {
    encoding: String,
}

impl MehdiNizarOriginalEncoder {
    pub fn new() -> MehdiNizarOriginalEncoder {
        return MehdiNizarOriginalEncoder {
            encoding: String::new(),
        };
    }
}

impl ILPEncoder for MehdiNizarOriginalEncoder {
    fn encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        _lower_bounds: usize,
        makespan: usize,
        timeout: &crate::common::timeout::Timeout,
    ) -> bool {
        let mut possible_makespans_at_decision = bitvec![0;makespan+1];
        possible_makespans_at_decision.set(0, true);

        let mut job_choices_at_node: Vec<Vec<usize>> = vec![Vec::new(); makespan + 1];

        for job in 0..partial_solution.instance.num_jobs {
            let mut possible_makespans_at_next_decision = bitvec![0;makespan+1];
            for i in 0..makespan + 1 {
                let job_size = partial_solution.instance.job_sizes[job];
                if possible_makespans_at_decision[i] == true {
                    possible_makespans_at_next_decision.set(i, true);
                    if i + job_size <= makespan {
                        possible_makespans_at_next_decision.set(i + job_size, true);
                        job_choices_at_node[i].push(job);
                    }
                }
            }
            possible_makespans_at_decision = possible_makespans_at_next_decision;
        }

        // Constraint 1
        let mut formula: String = String::from("Minimize\nmakespan\nSubject To\n");

        // Extra Constraint not in paper to improve runtime
        // formula += &format!("makespan >= {}\n", lower_bounds);

        // constraint 2
        for i in 0..makespan + 1 {
            for job in &job_choices_at_node[i] {
                let node_reached = i + partial_solution.instance.job_sizes[*job];
                if node_reached > makespan {
                    panic!("AAAAAAAAAAAAAAAAAAAAAAHHAHAHAHAHAHHAHAHAHAHAH");
                }
                formula += &format!("makespan - {} v_{}_{} >= 0\n", node_reached, i, job);
            }
            if timeout.time_finished() || formula.len() > 10_000_000_000 {
                return false;
            }
        }

        // Constraint 3
        let mut constraint_3 = String::from("v_0_0 ");
        for job in 1..job_choices_at_node[0].len() {
            constraint_3 += &format!("+ v_0_{} ", job_choices_at_node[0][job]);
        }
        constraint_3 += &format!("= {}\n", partial_solution.instance.num_processors);
        formula += &constraint_3;

        // constraint 4
        let mut in_edges: Vec<Vec<String>> = vec![Vec::new(); makespan + 1];

        for i in 0..makespan + 1 {
            for job in &job_choices_at_node[i] {
                if i + partial_solution.instance.job_sizes[*job] <= makespan {
                    in_edges[i + partial_solution.instance.job_sizes[*job]]
                        .push(format!("v_{}_{}", i, *job));
                }
            }
            if timeout.time_finished() || formula.len() > 10_000_000_000 {
                return false;
            }
        }

        for i in 1..makespan {
            let mut constraint = String::new();
            for job in &job_choices_at_node[i] {
                constraint += &format!("- v_{}_{} ", i, *job);
            }

            for edge in &in_edges[i] {
                constraint += &format!("+ {} ", edge);
            }
            if constraint.is_empty() {
                continue;
            }
            let constraint = format!("-f_{} {} = 0\n", i, constraint);
            formula += &constraint;
            if timeout.time_finished() || formula.len() > 10_000_000_000{
                return false;
            }
        }

        // constraint 5
        for job in 0..partial_solution.instance.num_jobs {
            let mut constraint = format!("v_0_{}", job);
            for i in 1..makespan + 1 {
                if job_choices_at_node[i].contains(&job) {
                    constraint += &format!(" + v_{}_{}", i, job);
                }
            }
            constraint += " = 1\n";
            formula += &constraint;
            if timeout.time_finished() || formula.len() > 10_000_000_000{
                return false;
            }
        }

        formula += "Binaries\n";

        for i in 0..makespan + 1 {
            for job in &job_choices_at_node[i] {
                formula += &format!(" v_{}_{}", i, *job);
            }
        }

        formula += "\nGenerals\nmakespan";

        for i in 0..makespan {
            formula += &format!(" f_{}", i);
        }
        formula += "\nEnd\n";

        self.encoding = formula;
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
