
use super::{
    basic_encoder::BasicEncoder,
    encoder::{Clause, Encoder}, binary_arithmetic,
};

pub struct BasicWithFillUp {
    pub basic: BasicEncoder,
    pub has_already_been_assigned_vars: Vec<Vec<Option<usize>>>,
    pub clauses: Vec<Clause>,
}

impl BasicWithFillUp {
    pub fn new() -> BasicWithFillUp {
        return BasicWithFillUp {
            basic: BasicEncoder::new(),
            has_already_been_assigned_vars: vec![],
            clauses: vec![],
        };
    }
}

impl Encoder for BasicWithFillUp {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
    ) {
        self.basic.basic_encode(partial_solution, makespan);
        let mut clauses: Vec<Clause> = vec![];
        let mut already_assigned_vars: Vec<Vec<Option<usize>>> = vec![];
        for job in 0..partial_solution.instance.num_jobs {
            let mut already_assigned_vars_i: Vec<Option<usize>> = vec![self.basic.problem.position_vars[job][0]];
            let mut previous_already_assigned_var: Option<usize> = self.basic.problem.position_vars[job][0];
            for processor in 1..partial_solution.instance.num_processors {
                // now we generate the vars
                if self.basic.problem.position_vars[job][processor].is_none() {
                    // if a job can't be assigned, there is no need to generate a new variable
                    already_assigned_vars_i.push(previous_already_assigned_var);
                } else {
                    // if a job can be assigned, generate a new var for it.
                    let next_already_assigned_var: Option<usize> = Some(self.basic.problem.var_name_generator.next());
                    if next_already_assigned_var.is_some() {
                        // assert that if you place it here the PAV will be true, 
                        clauses.push(Clause {
                            vars: vec![
                                -(self.basic.problem.position_vars[job][processor].unwrap() as i32),
                                (next_already_assigned_var.unwrap() as i32),
                            ],
                        });
                        if previous_already_assigned_var.is_some() {
                            // assert that the job was assigned to processor <= k-1 ==> the job is assigned to processor <= k
                            clauses.push(Clause {
                                vars: vec![
                                    -(previous_already_assigned_var.unwrap() as i32),
                                    (next_already_assigned_var.unwrap() as i32),
                                ],
                            });
                            // to avoid having no consequences to setting the PAV to true, we say that the previous_already_assigned_var prevents us from placing the job here
                            clauses.push(Clause {
                                vars: vec![
                                    -(previous_already_assigned_var.unwrap() as i32),
                                    -(self.basic.problem.position_vars[job][processor].unwrap() as i32),
                                ],
                            });
                        }
                    }
                    previous_already_assigned_var = next_already_assigned_var.clone();
                    already_assigned_vars_i.push(next_already_assigned_var);
                }
            }
            already_assigned_vars.push(already_assigned_vars_i);
        }

        // now we implement the fill up rule
        for job in 0..partial_solution.instance.num_jobs {
            for processor in 0..partial_solution.instance.num_processors {
                if already_assigned_vars[job][processor].is_some() && self.basic.problem.position_vars[job][processor].is_some() {
                    // if during the calculation of the partial the value is equal to the weight of this job, then this job must be placed here

                    let goal_sum = makespan - partial_solution.instance.job_sizes[job];
                    if goal_sum > self.basic.partial_sum_variables[processor][job].as_ref().unwrap().max {
                        continue;
                    }

                    let mut ne_clause = binary_arithmetic::not_equals_constant_encoding(self.basic.partial_sum_variables[processor][job].as_ref().unwrap(), makespan - partial_solution.instance.job_sizes[job]);
                    ne_clause.vars.push(self.has_already_been_assigned_vars[job][processor].unwrap() as i32);
                    clauses.push(ne_clause);
                }
            }
        }

        self.has_already_been_assigned_vars = already_assigned_vars;
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
