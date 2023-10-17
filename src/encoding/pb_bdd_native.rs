use crate::{
    common,
    problem_instance::{problem_instance::ProblemInstance, solution::Solution}, bdd,
};

use super::{
    binary_arithmetic::{self},
    encoder::{Clause, Encoder, VarNameGenerator},
};

pub struct PbNativeEncoder {
    pub var_name_generator: VarNameGenerator,
    pub clauses: Vec<Clause>,
    pub position_vars: Vec<Vec<Option<usize>>>,
}

impl PbNativeEncoder {
    pub fn new() -> PbNativeEncoder {
        return PbNativeEncoder {
            var_name_generator: VarNameGenerator::new(),
            clauses: vec![],
            position_vars: vec![],
        };
    }
}

impl Encoder for PbNativeEncoder {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
    ) {
        self.var_name_generator = VarNameGenerator::new();
        let mut clauses: Vec<Clause> = vec![];
        // here the presence of a variable indicates that process i can be put on server j
        let mut position_variables: Vec<Vec<Option<usize>>> = vec![];
        for i in 0..partial_solution.instance.num_jobs {
            let mut position_vars_i: Vec<Option<usize>> = vec![];
            for j in 0..partial_solution.instance.num_processors {
                if partial_solution.possible_allocations[i].contains(&j) {
                    position_vars_i.push(Some(self.var_name_generator.next()));
                } else {
                    position_vars_i.push(None)
                }
            }
            clauses.push(binary_arithmetic::at_least_one_encoding(
                position_vars_i
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap() as i32)
                    .collect(),
            ));
            clauses.append(&mut binary_arithmetic::pairwise_encoded_at_most_one(
                &position_vars_i
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap() as i32)
                    .collect(),
            ));
            position_variables.push(position_vars_i);

            
        }

       // for each processor, collect the vars that can go on it, and their weights, and build a bdd
       for proc in 0..partial_solution.instance.num_processors {
            let mut job_vars: Vec<usize> = vec![];
            let mut weights: Vec<usize> = vec![];
            for job in 0..partial_solution.instance.num_jobs{
                if position_variables[job][proc].is_some() {
                    job_vars.push(position_variables[job][proc].unwrap());
                    weights.push(partial_solution.instance.job_sizes[job]);
                }
            }
            // now we construct the bdd to assert that this machine is not too full
            let bdd = bdd::bdd::leq(&job_vars, &weights, makespan);
            let bdd = bdd::bdd::assign_aux_vars(bdd, &mut self.var_name_generator);
            let mut a = bdd::bdd::encode(&bdd);
            //println!("{:?}", a);
            clauses.append(&mut a);
       }
        
        self.clauses = clauses;
        self.position_vars = position_variables;
    }

    fn output(&self) -> Vec<Clause> {
        return self.clauses.clone();
    }

    fn decode(
        &self,
        instance: &ProblemInstance,
        var_assignment: &Vec<i32>,
    ) -> crate::problem_instance::solution::Solution {
        let mut assignment: Vec<usize> = vec![];
        for job in 0..self.position_vars.len() {
            for process in 0..self.position_vars[job].len() {
                if self.position_vars[job][process].is_some() && var_assignment.contains(&(*self.position_vars[job][process].as_ref().unwrap() as i32))
                {
                    assignment.push(process);
                }
            }
        }
        assert_eq!(assignment.len(), self.position_vars.len());

        let makespan = common::common::calc_makespan(instance, &assignment);

        return Solution{makespan, assignment};
    }

    fn get_num_vars(&self) -> usize {
        return self.var_name_generator.peek();
    }
}
