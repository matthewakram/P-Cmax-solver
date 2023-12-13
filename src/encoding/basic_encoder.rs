use crate::{common::timeout::Timeout, problem_instance::problem_instance::ProblemInstance};

use super::{
    binary_arithmetic::{self, BinaryNumber},
    encoder::{Clauses, Encoder, OneHotEncoder},
    problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding},
};

#[derive(Clone)]
pub struct BasicEncoder {
    pub problem: OneHotProblemEncoding,
    pub clauses: Clauses,
    pub final_sum_vars: Vec<BinaryNumber>,
    pub weight_on_machine_vars: Vec<Vec<Option<BinaryNumber>>>,
    pub partial_sum_variables: Vec<Vec<Option<BinaryNumber>>>,
}

impl BasicEncoder {
    pub fn new() -> BasicEncoder {
        return BasicEncoder {
            problem: OneHotProblemEncoding::new(),
            clauses: Clauses::new(),
            final_sum_vars: vec![],
            weight_on_machine_vars: vec![],
            partial_sum_variables: vec![],
        };
    }
}

impl Encoder for BasicEncoder {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
        timeout: &Timeout,
        _: usize,
    ) -> bool {
        self.problem.encode(partial_solution);
        let mut clauses: Clauses = Clauses::new();

        let mut weight_on_machine_vars: Vec<Vec<Option<BinaryNumber>>> = vec![];
        for job in 0..partial_solution.instance.num_jobs {
            weight_on_machine_vars.push(vec![]);
            for processor in 0..partial_solution.instance.num_processors {
                if self.problem.position_vars[job][processor].is_some() {
                    weight_on_machine_vars[job].push(Some(BinaryNumber::new(
                        partial_solution.instance.job_sizes[job],
                        &mut self.problem.var_name_generator,
                    )));
                    clauses.add_many_clauses(&mut binary_arithmetic::n_implies_m_in_j_encoding(
                        self.problem.position_vars[job][processor].unwrap(),
                        &weight_on_machine_vars[job][processor].as_ref().unwrap(),
                        &vec![partial_solution.instance.job_sizes[job]],
                    ));
                } else if self.problem.position_vars[job][processor].is_none() {
                    weight_on_machine_vars[job].push(None);
                }
            }
            if timeout.time_finished() {
                return false;
            }
        }

        let mut sum: Option<BinaryNumber>;
        let mut final_sum_variables: Vec<BinaryNumber> = vec![];
        let max_bitlength = binary_arithmetic::number_bitlength(makespan);
        let mut partial_sum_variables: Vec<Vec<Option<BinaryNumber>>> = vec![];

        for processor in 0..partial_solution.instance.num_processors {
            sum = None;
            partial_sum_variables.push(vec![]);
            for job in 0..partial_solution.instance.num_jobs {
                if weight_on_machine_vars[job][processor].is_some() {
                    if sum.is_none() {
                        sum = weight_on_machine_vars[job][processor].clone();
                    } else {
                        let (next_sum, mut sum_clauses) = binary_arithmetic::bounded_sum_encoding(
                            sum.as_ref().unwrap(),
                            weight_on_machine_vars[job][processor].as_ref().unwrap(),
                            max_bitlength,
                            &mut self.problem.var_name_generator,
                        );

                        clauses.add_many_clauses(&mut sum_clauses);
                        sum = Some(next_sum);
                    }
                    partial_sum_variables[processor].push(sum.clone());
                } else {
                    partial_sum_variables[processor].push(None);
                }
            }
            clauses.add_many_clauses(&mut binary_arithmetic::at_most_k_encoding(
                sum.as_ref().unwrap(),
                makespan,
            ));
            final_sum_variables.push(sum.as_ref().unwrap().clone());
            if timeout.time_finished() {
                return false;
            }
        }

        self.clauses = clauses;
        self.final_sum_vars = final_sum_variables;
        self.weight_on_machine_vars = weight_on_machine_vars;
        self.partial_sum_variables = partial_sum_variables;

        return true;
    }

    fn output(&mut self) -> Clauses {
        let mut out: Clauses = Clauses::new();
        std::mem::swap(&mut out, &mut self.clauses);
        out.add_many_clauses(&mut self.problem.clauses);
        return out;
    }

    fn decode(
        &self,
        instance: &ProblemInstance,
        var_assignment: &Vec<i32>,
    ) -> crate::problem_instance::solution::Solution {
        return self.problem.decode(instance, var_assignment);
    }

    fn get_num_vars(&self) -> usize {
        return self.problem.get_num_vars();
    }
}

impl OneHot for BasicEncoder {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.problem.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for BasicEncoder {}
