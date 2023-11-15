use crate::{
    common::timeout::Timeout, encoding::encoder::Clause, input_output, solvers::solver::SatResult,
};

use super::super::solver::SatSolver;
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    time::Duration,
};
use timeout_readwrite::TimeoutReader;

pub struct Kissat {}

impl SatSolver for Kissat {
    fn solve(&self, clauses: &Vec<Clause>, num_vars: usize, timeout: &Timeout) -> SatResult {
        if timeout.time_finished() {
            return SatResult::timeout();
        }
        let mut child: std::process::Child = Command::new("./kissat")
            .arg("-q")
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        let formula = input_output::to_dimacs::to_dimacs(clauses, num_vars, timeout);
        let a = child.stdout.take();

        if formula.is_none() {
            child.kill().unwrap();
            child.wait().unwrap();
            return SatResult::timeout();
        }

        let formula = formula.unwrap();
        {
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(formula.as_bytes()).unwrap();
            stdin.flush().unwrap();
        }

        let time_remaining = timeout.remaining_time();
        if time_remaining <= 0.0 || time_remaining.is_nan() || time_remaining.is_infinite() {
            child.kill().unwrap();
            child.wait().unwrap();
            return SatResult::timeout();
        }
        let mut reader = TimeoutReader::new(a.unwrap(), Duration::from_secs_f64(time_remaining));

        let mut out = String::new();
        let res: Result<usize, std::io::Error> = reader.read_to_string(&mut out);

        child.kill().unwrap();
        child.wait().unwrap();
        if res.is_err() {
            return SatResult::timeout();
        }

        let mut solution: Vec<i32> = vec![];
        for var in out.split(&[' ', '\n'][..]) {
            let number = var.parse::<i32>();
            match number {
                Ok(ok) => solution.push(ok),
                Err(_) => {}
            }
        }

        return if solution.len() == 0 {
            SatResult::unsat()
        } else {
            SatResult::sat(solution)
        };
    }
}
