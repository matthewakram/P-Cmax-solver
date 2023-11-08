use crate::{
    bdd::{self, bdd::BDD},
    problem_instance::{
        partial_solution::{self, PartialSolution},
        problem_instance::ProblemInstance,
    },
};

use super::{
    encoder::{Clause, Encoder, OneHotEncoder},
    problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding},
};

pub struct PbInter {
    one_hot: OneHotProblemEncoding,
    pub clauses: Vec<Clause>,
    opt_all: bool,
}

impl PbInter {
    pub fn new() -> PbInter {
        return PbInter {
            one_hot: OneHotProblemEncoding::new(),
            clauses: vec![],
            opt_all: true,
        };
    }

    pub fn _new_unopt() -> PbInter {
        return PbInter {
            one_hot: OneHotProblemEncoding::new(),
            clauses: vec![],
            opt_all: false,
        };
    }

    // this returns all of the vars of the nodes that represent the fur situation
    fn get_fur_vars(
        &self,
        bdd1: &BDD,
        solution: &PartialSolution,
        makespan_to_check: usize,
    ) -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = vec![];
        for i in &bdd1.nodes {
            if i.job_num == usize::MAX {
                continue;
            }
            let (lower, upper) = i.range;
            let job_size: usize = solution.instance.job_sizes[i.job_num];
            let fur_val = makespan_to_check - job_size;
            if lower <= fur_val && fur_val <= upper && lower == upper{
                //println!("job size {}, makespan {}, node_range {} {}", job_size, makespan_to_check, lower, upper);
                out.push((i.job_num, i.aux_var));
            }
        }
        return out;
    }
}

impl Encoder for PbInter {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
    ) {
        self.one_hot.encode(partial_solution);
        let mut clauses: Vec<Clause> = vec![];
        let mut bdds: Vec<BDD> = vec![];

        // for each processor, collect the vars that can go on it, and their weights, and build a bdd
        for proc in 0..partial_solution.instance.num_processors {
            let mut job_vars: Vec<usize> = vec![];
            let mut weights: Vec<usize> = vec![];
            let mut jobs: Vec<usize> = vec![];
            for job in 0..partial_solution.instance.num_jobs {
                if self.one_hot.position_vars[job][proc].is_some() && partial_solution.possible_allocations[job].len() > 1 
                {
                    job_vars.push(self.one_hot.position_vars[job][proc].unwrap());
                    jobs.push(job);
                    weights.push(partial_solution.instance.job_sizes[job]);
                }
            }
            if jobs.len() == 0 {
                bdds.push(BDD{ nodes: vec![], root_num: 0 })
            }else {
            // now we construct the bdd to assert that this machine is not too full
            let bdd = bdd::bdd::leq(&jobs, &job_vars, &weights, makespan, true, partial_solution.assigned_makespan[proc]);
            let bdd = bdd::bdd::assign_aux_vars(bdd, &mut self.one_hot.var_name_generator);
            let mut a: Vec<Clause> = bdd::bdd::_encode_bad(&bdd);

            //for n in &bdd.nodes {
            //    println!("proc {} var {} range {} {} ", n.job_num, n.aux_var, n.range.0, n.range.1);
            //}
            bdds.push(bdd);
            clauses.append(&mut a);
            }
        }

        for i in 0..partial_solution.instance.num_processors {
            for j in i + 1..partial_solution.instance.num_processors {
                if bdds[j].nodes.is_empty() || bdds[i].nodes.is_empty() {
                    continue;
                }
                clauses.append(&mut bdd::bdd::encode_bdd_bijective_relation(
                    &bdds[i], &bdds[j],
                ));
            }
        }

        if self.opt_all {
            // this encodes the fill up rule
            for i in 0..partial_solution.instance.num_processors - 1 {
                if bdds[i].nodes.is_empty() {
                    continue;
                }
                let fur_vars: Vec<(usize, usize)> =
                    self.get_fur_vars(&bdds[i], partial_solution, makespan);
                // TODO: test the performance difference between adding explicit fur nodes, and only considering final nodes with range
                // is of size one
                for (job_num, fur_var) in fur_vars {
                    for j in i + 1..partial_solution.instance.num_processors {
                        if self.one_hot.position_vars[job_num][j].is_some() {
                            clauses.push(Clause {
                                vars: vec![
                                    -(fur_var as i32),
                                    -(self.one_hot.position_vars[job_num][j].unwrap() as i32),
                                ],
                            })
                        }
                    }
                }
            }
        }

        self.clauses = clauses;
    }

    fn output(&self) -> Vec<Clause> {
        let mut out = self.clauses.clone();
        out.append(&mut self.one_hot.clauses.clone());
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

impl OneHot for PbInter {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.one_hot.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for PbInter {}
