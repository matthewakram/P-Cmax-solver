use std::collections::HashSet;

use crate::{
    common,
    encoding::{
        binary_arithmetic,
        encoder::{VarNameGenerator, Clauses},
    },
    problem_instance::{
        partial_solution::PartialSolution,
        problem_instance::ProblemInstance,
        solution::Solution,
    },
};

#[derive(Clone)]
pub struct OneHotProblemEncoding {
    pub var_name_generator: VarNameGenerator,
    pub position_vars: Vec<Vec<Option<usize>>>,
    pub clauses: Clauses,
}

pub trait OneHotClone : OneHot + Clone {}

pub trait OneHot {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize>;
}


impl OneHotProblemEncoding {
    pub fn new() -> OneHotProblemEncoding {
        let var_name_generator = VarNameGenerator::new();

        return OneHotProblemEncoding {
            var_name_generator,
            position_vars: vec![],
            clauses: Clauses::new(),
        };
    }

    pub fn encode(&mut self, partial_solution: &PartialSolution) {
        self.var_name_generator = VarNameGenerator::new();
        let mut clauses: Clauses = Clauses::new();
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
            clauses.add_clause(binary_arithmetic::at_least_one_encoding(
                position_vars_i
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap() as i32)
                    .collect(),
            ));
            // TODO: check the effect of using different at most one encodings
            clauses.add_many_clauses(&mut binary_arithmetic::pairwise_encoded_at_most_one(
                &position_vars_i
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap() as i32)
                    .collect(),
            ));
            position_variables.push(position_vars_i);
        }
        self.position_vars = position_variables;
        self.clauses = clauses;
    }

    pub fn decode(&self, instance: &ProblemInstance, var_assignment: &Vec<i32>) -> Solution {
        let mut assignment: Vec<usize> = vec![];
        // TODO: gotta improve this dramatically
        let mut assignment_set: HashSet<usize> = HashSet::new();
        for a in var_assignment {
            if *a >= 0 {
                assignment_set.insert(*a as usize);
            }
        }

        for job in 0..self.position_vars.len() {
            for process in 0..self.position_vars[job].len() {
                if self.position_vars[job][process].is_some()
                    && assignment_set
                        .contains(&self.position_vars[job][process].as_ref().unwrap())
                {
                    assignment.push(process);
                }
            }
        }
        assert_eq!(assignment.len(), self.position_vars.len());

        let makespan = common::common::calc_makespan(instance, &assignment);

        return Solution {
            makespan,
            assignment,
        };
    }

    pub fn get_num_vars(&self) -> usize {
        return self.var_name_generator.peek();
    }
}
