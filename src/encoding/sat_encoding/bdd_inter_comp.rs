use crate::{
    bdd::bdd_dyn::{self, DynBDD},
    common::timeout::Timeout,
    encoding::sat_encoder::{Clause, Clauses, Encoder, OneHotEncoder},
    problem_instance::problem_instance::ProblemInstance,
};

use super::problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding};

#[derive(Clone)]
pub struct BddInterComp {
    one_hot: OneHotProblemEncoding,
    pub clauses: Clauses,
    fur_rule: bool,
    inter_rule: bool,
}

impl BddInterComp {
    pub fn new() -> BddInterComp {
        return BddInterComp {
            one_hot: OneHotProblemEncoding::new(),
            clauses: Clauses::new(),
            fur_rule: true,
            inter_rule: true,
        };
    }

    pub fn new_inter_only() -> BddInterComp {
        return BddInterComp {
            one_hot: OneHotProblemEncoding::new(),
            clauses: Clauses::new(),
            fur_rule: false,
            inter_rule: true,
        };
    }

    pub fn new_basic() -> BddInterComp {
        return BddInterComp {
            one_hot: OneHotProblemEncoding::new(),
            clauses: Clauses::new(),
            fur_rule: false,
            inter_rule: false,
        };
    }
}

impl Encoder for BddInterComp {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
        timeout: &Timeout,
        max_num_clauses: usize,
    ) -> bool {
        self.one_hot.encode(partial_solution);
        let mut clauses: Clauses = Clauses::new();
        let mut bdds: Vec<DynBDD> = vec![];
        let undesignated_jobs: Vec<usize> = partial_solution
            .possible_allocations
            .iter()
            .enumerate()
            .filter(|(_, x)| x.len() > 1)
            .map(|(i, _)| i)
            .collect();
        let job_sizes: Vec<usize> = undesignated_jobs
            .iter()
            .map(|i| partial_solution.instance.job_sizes[*i])
            .collect();
        let range_table = bdd_dyn::RangeTable::new(&undesignated_jobs, &job_sizes, makespan);

        // for each processor, collect the vars that can go on it, and their weights, and build a bdd
        for proc in 0..partial_solution.instance.num_processors {
            let mut job_vars: Vec<usize> = vec![];
            let mut weights: Vec<usize> = vec![];
            let mut jobs: Vec<usize> = vec![];
            for job in 0..partial_solution.instance.num_jobs {
                if self.one_hot.position_vars[job][proc].is_some()
                    && partial_solution.possible_allocations[job].len() > 1
                {
                    job_vars.push(self.one_hot.position_vars[job][proc].unwrap());
                    jobs.push(job);
                    weights.push(partial_solution.instance.job_sizes[job]);
                }
            }
            if jobs.len() == 0 {
                bdds.push(DynBDD { nodes: vec![] })
            } else {
                // now we construct the bdd to assert that this machine is not too full
                let bdd = DynBDD::leq(
                    &jobs,
                    &job_vars,
                    &weights,
                    makespan,
                    partial_solution.assigned_makespan[proc],
                    &range_table,
                    timeout,
                );
                if bdd.is_none() {
                    return false;
                }
                let mut bdd = bdd.unwrap();
                bdd.assign_aux_vars(&mut self.one_hot.var_name_generator);
                let mut a: Clauses = bdd.encode();

                //for n in &bdd.nodes {
                //    println!("proc {} var {} range {} {} ", n.job_num, n.aux_var, n.range.0, n.range.1);
                //}
                bdds.push(bdd);
                clauses.add_many_clauses(&mut a);
            }

            if timeout.time_finished() || clauses.get_num_clauses() > max_num_clauses {
                return false;
            }
        }

        if self.inter_rule {
            for i in 0..partial_solution.instance.num_processors {
                for j in i + 1..partial_solution.instance.num_processors {
                    if bdds[j].nodes.is_empty() || bdds[i].nodes.is_empty() {
                        continue;
                    }
                    clauses.add_many_clauses(&mut bdds[i].encode_bdd_bijective_relation(&bdds[j]));
                }
                if timeout.time_finished() || clauses.get_num_clauses() > max_num_clauses {
                    return false;
                }
            }
        }

        if self.fur_rule {
            // this encodes the fill up rule
            for i in 0..partial_solution.instance.num_processors - 1 {
                if bdds[i].nodes.is_empty() {
                    continue;
                }
                let fur_vars: Vec<(usize, usize)> =
                    bdds[i].get_fur_vars(&range_table, partial_solution);
                for (job_num, fur_var) in fur_vars {
                    for j in i + 1..partial_solution.instance.num_processors {
                        if self.one_hot.position_vars[job_num][j].is_some() {
                            clauses.add_clause(Clause {
                                vars: vec![
                                    -(fur_var as i32),
                                    -(self.one_hot.position_vars[job_num][j].unwrap() as i32),
                                ],
                            })
                        }
                    }
                }
                if timeout.time_finished() || clauses.get_num_clauses() > max_num_clauses {
                    return false;
                }
            }
        }

        self.clauses = clauses;
        return true;
    }

    fn output(&mut self) -> Clauses {
        let mut out: Clauses = Clauses::new();
        std::mem::swap(&mut out, &mut self.clauses);
        out.add_many_clauses(&mut self.one_hot.clauses);
        return out;
    }

    fn decode(
        &self,
        instance: &ProblemInstance,
        var_assignment: &Vec<i32>,
    ) -> crate::problem_instance::solution::Solution {
        return self.one_hot.decode(instance, var_assignment);
    }

    fn get_num_vars(&self) -> usize {
        return self.one_hot.var_name_generator.peek();
    }
}

impl OneHot for BddInterComp {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.one_hot.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for BddInterComp {}
