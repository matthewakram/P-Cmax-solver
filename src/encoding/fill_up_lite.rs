
use super::{
    basic_encoder::BasicEncoder,
    encoder::{Clause, Encoder}, binary_arithmetic,
};

pub struct FillUpLite {
    pub basic: BasicEncoder,
    pub clauses: Vec<Clause>,
}

impl FillUpLite {
    pub fn new() -> FillUpLite {
        return FillUpLite {
            basic: BasicEncoder::new(),
            clauses: vec![],
        };
    }
}

impl Encoder for FillUpLite {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
    ) {
        self.basic.basic_encode(partial_solution, makespan);
        let mut clauses: Vec<Clause> = vec![];


        // now we implement the fill up rule
        for job in 0..partial_solution.instance.num_jobs {
            for processor in 0..partial_solution.instance.num_processors {
                if self.basic.position_vars[job][processor].is_some() {
                    // if during the calculation of the partial the value is equal to the weight of this job, then this job must be placed here
                    // in essence this means partial_sum_i == weight ==> pos_var_job_1 | pos_var_job_2 | ...| pos_var_job_i
                    // TODO this
                    let goal_sum = makespan - partial_solution.instance.job_sizes[job];
                    if goal_sum > self.basic.partial_sum_variables[processor][job].as_ref().unwrap().max {
                        continue;
                    }

                    let mut ne_clause = binary_arithmetic::not_equals_constant_encoding(self.basic.partial_sum_variables[processor][job].as_ref().unwrap(), goal_sum);
                    let mut previous_position_clauses: Vec<i32> = self.basic.position_vars[job].iter().enumerate().filter(|(i,x)| *i <= processor && x.is_some()).map(|(_, x)| (x.unwrap() as i32)).collect();
                    ne_clause.vars.append(&mut previous_position_clauses);

                    clauses.push(ne_clause);
                }
            }
        }

        self.clauses = clauses;
    }

    fn output(&self) -> Vec<Clause> {
        let mut out = self.clauses.clone();
        out.append(&mut self.basic.output());
        return out;
    }

    fn decode(
        &self,
        instance: &crate::problem_instance::problem_instance::ProblemInstance,
        solution: &Vec<i32>,
    ) -> crate::problem_instance::solution::Solution {
        let sol = self.basic.decode(instance, solution);
        //println!("already assigned vars: ");
        //for job in 0..self.has_already_been_assigned_vars.len()  {
        //    for process in 0..self.has_already_been_assigned_vars[job].len() {
        //        if self.has_already_been_assigned_vars[job][process].is_some() {
        //            print!("{},", solution.contains(&(*self.has_already_been_assigned_vars[job][process].as_ref().unwrap() as i32)));
        //        }
        //    }
        //    println!("");
        //}
        //println!("{}", sol);
        return sol;
    }

    fn get_num_vars(&self) -> usize {
        return self.basic.get_num_vars();
    }
}
