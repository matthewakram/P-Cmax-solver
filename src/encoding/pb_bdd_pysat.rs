use crate::{
    common,
    problem_instance::{problem_instance::ProblemInstance, solution::Solution},
};
use std::{fs::{File, read_to_string}, io::Write, process::Command};


use super::{
    binary_arithmetic::{self},
    encoder::{Clause, Encoder, VarNameGenerator},
};

pub struct PbPysatEncoder {
    pub var_name_generator: VarNameGenerator,
    pub clauses: Vec<Clause>,
    pub position_vars: Vec<Vec<Option<usize>>>,
}

impl PbPysatEncoder {
    pub fn new() -> PbPysatEncoder {
        return PbPysatEncoder {
            var_name_generator: VarNameGenerator::new(),
            clauses: vec![],
            position_vars: vec![],
        };
    }
}

impl Encoder for PbPysatEncoder {
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

        let mut file = File::create("./pysat_file").unwrap();
        file.write(format!("{} {} {} {}\n", partial_solution.instance.num_jobs, partial_solution.instance.num_processors, self.get_num_vars(), makespan).as_bytes()).unwrap();
        let string = partial_solution.instance.job_sizes.iter().map(|x| x.to_string() + " ").reduce(|x,y| x + &y).unwrap();
        file.write(string.as_bytes()).unwrap();
        file.write("\n".as_bytes()).unwrap();
        for job in 0..partial_solution.instance.num_jobs {
            let mut string: String = String::new();
            for proc in 0..partial_solution.instance.num_processors {
                string += &(position_variables[job][proc].as_ref().unwrap_or(&0)).to_string();
                string += " ";
            }
            string += "\n";
            file.write(string.as_bytes()).unwrap();
        }

        Command::new("python3")
                     .arg("./src/encoding/pb_with_pysat.py")
                     .status()
                     .expect("failed to execute process");
        
        let binding = read_to_string("./pysat_file_1").unwrap();
        let reader = binding.lines();
        for line in reader {
            let line = line.split(" ");
            clauses.push(Clause{vars: line.map(|x| x.parse::<i32>().unwrap()).collect::<Vec<i32>>()})
        }

        let last_var = clauses.last().as_ref().unwrap().vars[0].abs() as usize;
        self.var_name_generator.jump_to(last_var + 1);
        
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
