use crate::{bdd, problem_instance::problem_instance::ProblemInstance};

use super::{
    encoder::{Clause, Encoder, OneHotEncoder},
    problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding},
};

pub struct PbNativeEncoderFE {
    one_hot: OneHotProblemEncoding,
    pub clauses: Vec<Clause>,
}

impl PbNativeEncoderFE {
    pub fn new() -> PbNativeEncoderFE {
        return PbNativeEncoderFE {
            one_hot: OneHotProblemEncoding::new(),
            clauses: vec![],
        };
    }
}

impl Encoder for PbNativeEncoderFE {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
    ) {
        self.one_hot.encode(partial_solution);
        let mut clauses: Vec<Clause> = vec![];

        // for each processor, collect the vars that can go on it, and their weights, and build a bdd
        for proc in 0..partial_solution.instance.num_processors {
            let mut job_vars: Vec<usize> = vec![];
            let mut weights: Vec<usize> = vec![];
            for job in 0..partial_solution.instance.num_jobs {
                if self.one_hot.position_vars[job][proc].is_some() {
                    job_vars.push(self.one_hot.position_vars[job][proc].unwrap());
                    weights.push(partial_solution.instance.job_sizes[job]);
                }
            }
            // now we construct the bdd to assert that this machine is not too full
            let bdd = if proc < partial_solution.instance.num_processors / 2 {
                let tbdd = bdd::bdd::eq(&job_vars, &weights, makespan);
                if tbdd.is_none() {
                    self.clauses = vec![Clause { vars: vec![1] }, Clause { vars: vec![-1] }];
                    return;
                }
                tbdd.unwrap()
            } else {
                bdd::bdd::leq(&job_vars, &weights, makespan)
            };

            let bdd = bdd::bdd::assign_aux_vars(bdd, &mut self.one_hot.var_name_generator);
            let mut a = if proc < partial_solution.instance.num_processors / 2 {
                bdd::bdd::encode_bad(&bdd)
            } else {
                bdd::bdd::encode(&bdd)
            };

            //println!("{:?}", a);
            clauses.append(&mut a);
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

impl OneHot for PbNativeEncoderFE {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.one_hot.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for PbNativeEncoderFE {}
