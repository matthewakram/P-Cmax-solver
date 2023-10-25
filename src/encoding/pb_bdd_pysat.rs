use crate::problem_instance::problem_instance::ProblemInstance;
use std::{
    fs::{read_to_string, File},
    io::Write,
    process::Command,
};

use super::{
    encoder::{Clause, Encoder, OneHotEncoder},
    problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding},
};

pub struct PbPysatEncoder {
    one_hot: OneHotProblemEncoding,
    pub clauses: Vec<Clause>,
}

impl PbPysatEncoder {
    pub fn new() -> PbPysatEncoder {
        return PbPysatEncoder {
            one_hot: OneHotProblemEncoding::new(),
            clauses: vec![],
        };
    }
}

impl Encoder for PbPysatEncoder {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
    ) {
        self.one_hot.encode(partial_solution);
        let mut clauses: Vec<Clause> = vec![];

        let mut file = File::create("./pysat_file").unwrap();
        file.write(
            format!(
                "{} {} {} {}\n",
                partial_solution.instance.num_jobs,
                partial_solution.instance.num_processors,
                self.get_num_vars(),
                makespan
            )
            .as_bytes(),
        )
        .unwrap();
        let string = partial_solution
            .instance
            .job_sizes
            .iter()
            .map(|x| x.to_string() + " ")
            .reduce(|x, y| x + &y)
            .unwrap();
        file.write(string.as_bytes()).unwrap();
        file.write("\n".as_bytes()).unwrap();
        for job in 0..partial_solution.instance.num_jobs {
            let mut string: String = String::new();
            for proc in 0..partial_solution.instance.num_processors {
                string +=
                    &(self.one_hot.position_vars[job][proc].as_ref().unwrap_or(&0)).to_string();
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
        let mut max: usize = 0;
        for line in reader {
            let line = line.split(" ");
            clauses.push(Clause {
                vars: line
                    .clone()
                    .map(|x| x.parse::<i32>().unwrap())
                    .collect::<Vec<i32>>(),
            });
            max = max.max(
                line.map(|x| x.parse::<i32>().unwrap().abs() as usize)
                    .max()
                    .unwrap_or(0),
            );
        }

        self.one_hot.var_name_generator.jump_to(max + 1);
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

impl OneHot for PbPysatEncoder {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.one_hot.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for PbPysatEncoder {}
