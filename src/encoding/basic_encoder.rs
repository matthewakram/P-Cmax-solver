use crate::{
    common,
    problem_instance::{problem_instance::ProblemInstance, solution::Solution},
};

use super::{
    binary_arithmetic::{self, BinaryNumber},
    encoder::{Clause, Encoder, VarNameGenerator},
};

pub struct BasicEncoder {
    pub var_name_generator: VarNameGenerator,
    pub clauses: Vec<Clause>,
    pub position_vars: Vec<Vec<Option<usize>>>,
    pub final_sum_vars: Vec<BinaryNumber>,
    pub weight_on_machine_vars: Vec<Vec<Option<BinaryNumber>>>,
    pub partial_sum_variables: Vec<Vec<Option<BinaryNumber>>>
}

impl BasicEncoder {
    pub fn new() -> BasicEncoder {
        return BasicEncoder {
            var_name_generator: VarNameGenerator::new(),
            clauses: vec![],
            position_vars: vec![],
            final_sum_vars: vec![],
            weight_on_machine_vars: vec![],
            partial_sum_variables: vec![]
        };
    }
}

impl Encoder for BasicEncoder {
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
            // TODO: check the effect of using different at most one encodings
            clauses.append(&mut binary_arithmetic::pairwise_encoded_at_most_one(
                &position_vars_i
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap() as i32)
                    .collect(),
            ));
            position_variables.push(position_vars_i);
        }

        let mut weight_on_machine_vars: Vec<Vec<Option<BinaryNumber>>> = vec![];
        for job in 0..position_variables.len() {
            weight_on_machine_vars.push(vec![]);
            for processor in 0..position_variables[0].len() {
                if position_variables[job][processor].is_some() {
                    weight_on_machine_vars[job].push(Some(BinaryNumber::new(
                        partial_solution.instance.job_sizes[job],
                        &mut self.var_name_generator,
                    )));
                    clauses.append(&mut binary_arithmetic::n_implies_m_in_j_encoding(
                        position_variables[job][processor].unwrap(),
                        &weight_on_machine_vars[job][processor].as_ref().unwrap(),
                        &vec![partial_solution.instance.job_sizes[job]],
                    ));
                } else if position_variables[job][processor].is_none() {
                    weight_on_machine_vars[job].push(None);
                }
            }
        }

        let mut sum: Option<BinaryNumber>;
        let mut final_sum_variables: Vec<BinaryNumber> = vec![];
        let max_bitlength = binary_arithmetic::number_bitlength(makespan);
        let mut partial_sum_variables: Vec<Vec<Option<BinaryNumber>>> = vec![];

        for processor in 0..position_variables[0].len() {
            sum = None;
            partial_sum_variables.push(vec![]);
            for job in 0..position_variables.len() {
                if weight_on_machine_vars[job][processor].is_some() {
                    if sum.is_none() {
                        sum = weight_on_machine_vars[job][processor].clone();
                    } else {
                        let (next_sum, mut sum_clauses) = binary_arithmetic::bounded_sum_encoding(
                            sum.as_ref().unwrap(),
                            weight_on_machine_vars[job][processor].as_ref().unwrap(),
                            max_bitlength,
                            &mut self.var_name_generator,
                        );

                        clauses.append(&mut sum_clauses);
                        sum = Some(next_sum);
                    }
                    partial_sum_variables[processor].push(sum.clone());
                } else {
                    partial_sum_variables[processor].push(None);
                }
            }
            clauses.append(&mut binary_arithmetic::at_most_k_encoding(
                sum.as_ref().unwrap(),
                makespan,
            ));
            final_sum_variables.push(sum.as_ref().unwrap().clone());
        }

        self.clauses = clauses;
        self.position_vars = position_variables;
        self.final_sum_vars = final_sum_variables;
        self.weight_on_machine_vars = weight_on_machine_vars;
        self.partial_sum_variables = partial_sum_variables;
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

        //print!("sum_vars: ");
        //for sum_var in &self.final_sum_vars{
        //    print!("{}, ", sum_var.from_assignment(var_assignment))
        //}
        //print!("more sum vars \n");
        //for i in 0..self.weight_on_machine_vars.len(){
        //    
        //    for j in 0..self.weight_on_machine_vars[0].len(){
        //        if self.weight_on_machine_vars[i][j].is_some() {
        //            print!("{}, ", self.weight_on_machine_vars[i][j].as_ref().unwrap().from_assignment(var_assignment));
        //        }
        //    }
        //    print!("\n");
        //}
        //print!("\n");
//
        //print!("more sum sum vars \n");
        //println!("{:?}", self.partial_sum_variables);
        //for i in 0..self.partial_sum_variables.len(){
        //    for j in 0..self.partial_sum_variables[i].len(){
        //        print!("{}, ", self.partial_sum_variables[i][j].from_assignment(var_assignment));
        //    }
        //    print!("\n");
        //}
        //print!("\n");

        return Solution{makespan, assignment};

    }

    fn get_num_vars(&self) -> usize {
        return self.var_name_generator.peek();
    }
}
