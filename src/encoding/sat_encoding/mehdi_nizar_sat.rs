use std::collections::HashSet;

use crate::{
    common::{self, timeout::Timeout},
    encoding::sat_encoder::{Clause, Clauses, Encoder, VarNameGenerator},
    problem_instance::{problem_instance::ProblemInstance, solution::Solution},
};

use super::{binary_arithmetic, cardinality_networks};

use bitvec::prelude::*;
#[derive(Clone)]
pub struct MehdiNizarSatEncoder {
    var_name_generator: VarNameGenerator,
    vars: Vec<Vec<usize>>,
    clauses: Clauses,
}

impl MehdiNizarSatEncoder {
    pub fn new() -> MehdiNizarSatEncoder {
        return MehdiNizarSatEncoder {
            var_name_generator: VarNameGenerator::new(),
            vars: vec![],
            clauses: Clauses::new(),
        };
    }
}

impl Encoder for MehdiNizarSatEncoder {
    // TODO add timeout to encode
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
        _timeout: &Timeout,
        _max_num_clauses: usize,
    ) -> bool {
        let mut possible_makespans_at_decision = bitvec![0;makespan+1];
        possible_makespans_at_decision.set(0, true);
        self.var_name_generator = VarNameGenerator::new();

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
        drop(possible_makespans_at_decision);

        let mut vars: Vec<Vec<usize>> = vec![vec![0; partial_solution.instance.num_jobs]; makespan];
        for i in 0..makespan {
            for job in &job_choices_at_node[i] {
                vars[i][*job] = self.var_name_generator.next();
            }
        }

        let mut clauses = Clauses::new();

        // Constraint 1

        // Extra Constraint not in paper to improve runtime
        //formula += &format!("makespan >= {}\n", lower_bounds);

        // constraint 2
        for i in 0..makespan + 1 {
            for job in &job_choices_at_node[i] {
                let node_reached = i + partial_solution.instance.job_sizes[*job];
                if node_reached > makespan {
                    panic!("AAAAAAAAAAAAAAAAAAAAAAHHAHAHAHAHAHHAHAHAHAHAH");
                }
                //formula += &format!("makespan - {} v_{}_{} >= 0\n", node_reached, i, job);
            }
        }

        // Constraint 3
        // The number of jobs assigned at time 0 is exactly num_procs
        let vars_on_node_0: Vec<usize> = vars[0].iter().filter(|x| **x != 0).map(|x| *x).collect();
        let (mut card_clauses, num_true_vars) = cardinality_networks::basic_sort(
            &vars_on_node_0,
            partial_solution.instance.num_processors,
            &mut self.var_name_generator,
        );
        clauses.add_many_clauses(&mut card_clauses);
        clauses.add_clause(Clause {
            vars: vec![*num_true_vars.last().unwrap() as i32],
        });

        // constraint 4
        let mut in_edges: Vec<Vec<usize>> = vec![Vec::new(); makespan + 1];

        for i in 0..makespan + 1 {
            for job in &job_choices_at_node[i] {
                if i + partial_solution.instance.job_sizes[*job] <= makespan {
                    in_edges[i + partial_solution.instance.job_sizes[*job]].push(vars[i][*job]);
                } else {
                    panic!("hmm why did I think I cant reach here");
                }
            }
        }

        for i in 1..makespan {
            if in_edges[i].len() == 0 || job_choices_at_node[i].len() == 0 {
                continue;
            }

            let (mut in_card_clauses, in_var_card) = cardinality_networks::basic_sort(
                &in_edges[i],
                partial_solution.instance.num_processors,
                &mut self.var_name_generator,
            );
            clauses.add_many_clauses(&mut in_card_clauses);
            let out_vars: Vec<usize> = vars[i]
                .iter()
                .filter(|x| **x != 0)
                .map(|x: &usize| *x)
                .collect();
            let max_num_vars = out_vars
                .len()
                .min(in_edges[i].len())
                .min(partial_solution.instance.num_processors);
            let (mut out_card_clauses, out_var_card) = cardinality_networks::basic_sort(
                &out_vars,
                max_num_vars,
                &mut self.var_name_generator,
            );
            clauses.add_many_clauses(&mut out_card_clauses);

            for j in 0..out_var_card.len() {
                clauses.add_clause(Clause {
                    vars: vec![-(out_var_card[j] as i32), in_var_card[j] as i32],
                });
            }
        }

        // constraint 5
        for job in 0..partial_solution.instance.num_jobs {
            let edges_belonging_to_job: Vec<i32> = vars
                .iter()
                .map(|i| i[job])
                .filter(|var| *var != 0)
                .map(|x| x as i32)
                .collect();
            clauses.add_clause(binary_arithmetic::at_least_one_encoding(
                edges_belonging_to_job.clone(),
            ));
            clauses.add_many_clauses(&mut binary_arithmetic::pairwise_encoded_at_most_one(
                &edges_belonging_to_job,
            ));
        }

        self.clauses = clauses;
        self.vars = vars;
        return true;
    }

    fn output(&mut self) -> Clauses {
        let mut out: Clauses = Clauses::new();
        std::mem::swap(&mut out, &mut self.clauses);
        return out;
    }

    fn decode(
        &self,
        instance: &ProblemInstance,
        var_assignment: &Vec<i32>,
    ) -> crate::problem_instance::solution::Solution {
        let mut assignment: Vec<usize> = vec![0; instance.num_jobs];
        let mut unassigned_jobs: HashSet<usize> = (0..instance.num_jobs).collect();
        let var_assignment: HashSet<usize> = var_assignment
            .iter()
            .filter(|x| **x > 0)
            .map(|x| *x as usize)
            .collect();

        for proc in 0..instance.num_processors {
            let mut current_node = 0;
            while current_node < self.vars.len() {
                let old_current = current_node;
                for job in 0..instance.num_jobs {
                    if !unassigned_jobs.contains(&job) {
                        continue;
                    }

                    let assigned = var_assignment.contains(&self.vars[current_node][job]);

                    if assigned {
                        unassigned_jobs.remove(&job);
                        assignment[job] = proc;
                        current_node += instance.job_sizes[job];
                        break;
                    }
                }
                if current_node == old_current {
                    break;
                }
            }
        }

        let makespan_check = common::common::calc_makespan(instance, &assignment);

        return Solution {
            makespan: makespan_check,
            assignment,
        };
    }

    fn get_num_vars(&self) -> usize {
        return self.var_name_generator.peek();
    }
}
