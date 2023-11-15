use timeout_readwrite::TimeoutReader;

use crate::{common::timeout::Timeout, problem_instance::problem_instance::ProblemInstance};
use std::{
    io::{BufWriter, Read, Write},
    process::{Command, Stdio},
    time::Duration,
};

use super::{
    encoder::{Clause, Encoder, OneHotEncoder},
    problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding},
};

#[derive(Clone)]
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
    // TODO add timeout to encode
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
        timeout: &Timeout,
    ) -> bool {
        let mut child;
        loop {
            let res = Command::new("python3")
                .arg("./src/encoding/pb_with_pysat.py")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn();
            if res.is_ok() {
                child = res.unwrap();
                break;
            }
        }
        self.one_hot.encode(partial_solution);
        let mut clauses: Vec<Clause> = vec![];

        let mut string = String::new();
        string += format!(
            "{} {} {} {}\n",
            partial_solution.instance.num_jobs,
            partial_solution.instance.num_processors,
            self.get_num_vars(),
            makespan
        )
        .as_str();

        string += partial_solution
            .instance
            .job_sizes
            .iter()
            .map(|x| x.to_string() + " ")
            .reduce(|x, y| x + &y)
            .unwrap()
            .as_str();
        string += "\n";

        for job in 0..partial_solution.instance.num_jobs {
            for proc in 0..partial_solution.instance.num_processors {
                string +=
                    &(self.one_hot.position_vars[job][proc].as_ref().unwrap_or(&0)).to_string();
                string += " ";
            }
            string += "\n";
        }

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let mut writer = BufWriter::new(stdin);
        writer.write_all(string.as_bytes()).unwrap();
        writer.flush().unwrap();

        let time_remaining = timeout.remaining_time();
        if time_remaining <= 0.0 || time_remaining.is_nan() || time_remaining.is_infinite() {
            child.kill().unwrap();
            return false;
        }

        let mut reader: TimeoutReader<std::process::ChildStdout> =
            TimeoutReader::new(stdout, Duration::from_secs_f64(time_remaining));

        let mut out = String::new();
        let error = reader.read_to_string(&mut out);

        child.kill().unwrap();
        child.wait().unwrap();
        if error.is_err() {
            return false;
        }

        let lines = out.lines();
        let mut max: usize = 0;
        for line in lines {
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

        max = max.max(self.one_hot.get_num_vars());

        self.one_hot.var_name_generator.jump_to(max + 1);
        self.clauses = clauses;
        return true;
    }

    fn output(&self) -> Vec<Clause> {
        let mut out: Vec<Clause> = self.clauses.clone();
        out.append(&mut self.one_hot.clauses.clone());
        //let num_vars = self.get_num_vars();
        //for i in &out {
        //    for v in &i.vars{
        //        if v.abs() as usize > num_vars {
        //            println!("error occured at {} {}", v, num_vars);
        //        }
        //        assert!(v.abs() as usize <= num_vars);
        //    }
        //}
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
