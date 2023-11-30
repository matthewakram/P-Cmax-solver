use crate::{bdd::{self, bdd::BDD}, problem_instance::problem_instance::ProblemInstance, common::timeout::Timeout};

use super::{
    encoder::{Encoder, OneHotEncoder, Clauses},
    problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding},
};

#[derive(Clone)]
pub struct PbNativeEncoder {
    one_hot: OneHotProblemEncoding,
    pub clauses: Clauses,
}

impl PbNativeEncoder {
    pub fn new() -> PbNativeEncoder {
        return PbNativeEncoder {
            one_hot: OneHotProblemEncoding::new(),
            clauses: Clauses::new(),
        };
    }
}

impl Encoder for PbNativeEncoder {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
        timeout: &Timeout,
        max_num_clauses: usize
    ) -> bool{
        self.one_hot.encode(partial_solution);
        let mut clauses= Clauses::new();

        // for each processor, collect the vars that can go on it, and their weights, and build a bdd
        for proc in 0..partial_solution.instance.num_processors {
            let mut job_vars: Vec<usize> = vec![];
            let mut weights: Vec<usize> = vec![];
            let mut jobs: Vec<usize> = vec![];
            for job in 0..partial_solution.instance.num_jobs {
                if self.one_hot.position_vars[job][proc].is_some() && partial_solution.possible_allocations[job].len() > 1 {
                    job_vars.push(self.one_hot.position_vars[job][proc].unwrap());
                    jobs.push(job);
                    weights.push(partial_solution.instance.job_sizes[job]);
                }
            }
            if jobs.len() == 0 {
                continue;
            }
            // now we construct the bdd to assert that this machine is not too full
            let bdd: Option<BDD> = bdd::bdd::leq(&jobs, &job_vars, &weights, makespan, false, partial_solution.assigned_makespan[proc], timeout);
            if bdd.is_none() {
                return false;
            }
            let bdd = bdd.unwrap();
            let bdd: BDD = bdd::bdd::assign_aux_vars(bdd, &mut self.one_hot.var_name_generator);
            

            //for i in 0..bdd.nodes.len(){
            //    let a = &bdd.nodes[i];
            //    println!("{}    var: {}, aux var: {} left {} right {}, left aux {} right aux {}", i, a.var, a.aux_var, a.left_child, a.right_child, bdd.nodes[a.left_child].aux_var, bdd.nodes[a.right_child].aux_var);
            //}
            let mut a: Clauses = bdd::bdd::encode(&bdd);

            //println!("{:?}", a);
            clauses.add_many_clauses(&mut a);
            if timeout.time_finished() {
                return false;
            }
            if clauses.get_num_clauses() > max_num_clauses {
                return false;
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

impl OneHot for PbNativeEncoder {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.one_hot.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for PbNativeEncoder {}
