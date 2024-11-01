use std::{
    io::{BufReader, BufWriter, Write},
    process::{Command, Stdio},
};

use crate::{problem_instance::solution::Solution, solvers::solver_manager::SolverManager};

#[derive(Clone)]
pub struct CpSatSolverManager {}

impl SolverManager for CpSatSolverManager {
    fn solve(
        &mut self,
        instance: &crate::problem_instance::problem_instance::ProblemInstance,
        lower: usize,
        upper: &crate::problem_instance::solution::Solution,
        timeout: &crate::common::timeout::Timeout,
        _verbose: bool,
    ) -> Option<crate::problem_instance::solution::Solution> {
        let mut child = Command::new("python3")
            .arg("./src/solvers/cp_sat/pcmax_cp_sat_solver.py")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut string = String::new();
        string += format!(
            "{} {} {} {} {}\n",
            instance.num_jobs,
            instance.num_processors,
            lower,
            upper.makespan,
            timeout.remaining_time()
        )
        .as_str();

        string += instance
            .job_sizes
            .iter()
            .map(|x| x.to_string() + " ")
            .reduce(|x, y| x + &y)
            .unwrap()
            .as_str();
        string += "\n";

        let stdin = child.stdin.take().unwrap();

        let mut writer = BufWriter::new(stdin);
        writer.write_all(string.as_bytes()).unwrap();
        writer.flush().unwrap();

        let output: std::process::Output = child.wait_with_output().unwrap();
        let output = String::from_utf8(output.stdout).unwrap();
        let output: Vec<&str> = output.split("\n").collect();

        if output[0] == "unsatisfiable" {
            return Some(upper.clone());
        } else if output[0] == "timeout" {
            return None;
        }

        let makespan: usize = output[0].parse().unwrap();
        let assignment: Vec<usize> = output[1]
            .split_whitespace()
            .map(|x| x.parse().unwrap())
            .collect();

        return Some(Solution {
            makespan,
            assignment,
        });
    }
}
