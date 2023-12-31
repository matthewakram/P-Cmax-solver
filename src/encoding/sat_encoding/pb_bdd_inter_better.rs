use crate::{
    bdd::{self, bdd::BDD},
    common::timeout::Timeout,
    problem_instance::{partial_solution::PartialSolution, problem_instance::ProblemInstance}, encoding::sat_encoder::{Clauses, Encoder, Clause, OneHotEncoder},
};

use super::problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding};

#[derive(Clone)]
pub struct PbInterDyn {
    one_hot: OneHotProblemEncoding,
    pub clauses: Clauses,
    opt_all: bool,
}

impl PbInterDyn {
    pub fn new() -> PbInterDyn {
        return PbInterDyn {
            one_hot: OneHotProblemEncoding::new(),
            clauses: Clauses::new(),
            opt_all: true,
        };
    }

    // this returns all of the vars of the nodes that represent the fur situation
    fn get_fur_vars(
        &self,
        bdd1: &BDD,
        solution: &PartialSolution,
        makespan_to_check: usize,
    ) -> Vec<(usize, usize)> {
        //println!("{}", makespan_to_check);
        //println!("{:?}", solution.instance.job_sizes);
        let mut jobs_in_bdd = vec![];
        for i in &bdd1.nodes {
            if i.job_num != usize::MAX
                && (jobs_in_bdd.len() == 0 || jobs_in_bdd[jobs_in_bdd.len() - 1] != i.job_num)
            {
                jobs_in_bdd.push(i.job_num);
            }
        }

        let mut out: Vec<(usize, usize)> = vec![];

        for i in 0..jobs_in_bdd.len() {
            let job = jobs_in_bdd[i];
            let next_job = if i != jobs_in_bdd.len() - 1 {
                jobs_in_bdd[i + 1]
            } else {
                usize::MAX
            };
            let fur_val = makespan_to_check - solution.instance.job_sizes[job];
            for node in &bdd1.nodes {
                if node.job_num == usize::MAX {
                    continue;
                }
                if node.job_num == next_job {
                    break;
                }
                let (lower, upper) = node.range;

                if lower <= fur_val && fur_val == upper {
                    out.push((job, node.aux_var));
                    //print!("({} {} {} {}) ", job, node.job_num, node.range.0, node.range.1);
                }
            }
        }
        //println!("");
        return out;
    }
}

impl Encoder for PbInterDyn {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
        timeout: &Timeout,
        max_num_clauses: usize,
    ) -> bool {
        self.one_hot.encode(partial_solution);
        let mut clauses: Clauses = Clauses::new();
        let mut bdds: Vec<BDD> = vec![];

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
                bdds.push(BDD { nodes: vec![] })
            } else {
                // now we construct the bdd to assert that this machine is not too full
                let bdd = bdd::bdd::leq(
                    &jobs,
                    &job_vars,
                    &weights,
                    makespan,
                    false,
                    partial_solution.assigned_makespan[proc],
                    timeout,
                );
                if bdd.is_none() {
                    return false;
                }
                let bdd = bdd.unwrap();
                let bdd = bdd::bdd::assign_aux_vars(bdd, &mut self.one_hot.var_name_generator);
                let mut a: Clauses = bdd::bdd::encode_bad(&bdd);

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

        for i in 0..partial_solution.instance.num_processors {
            for j in i + 1..partial_solution.instance.num_processors {
                if bdds[j].nodes.is_empty() || bdds[i].nodes.is_empty() {
                    continue;
                }
                clauses.add_many_clauses(&mut bdd::bdd::encode_bdd_bijective_relation(
                    &bdds[i], &bdds[j],
                ));
            }
            if timeout.time_finished() || clauses.get_num_clauses() > max_num_clauses {
                return false;
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

impl OneHot for PbInterDyn {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.one_hot.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for PbInterDyn {}
