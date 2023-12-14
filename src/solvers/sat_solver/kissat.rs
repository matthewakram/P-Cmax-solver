use crate::{
    common::timeout::Timeout, encoding::encoder::Clauses, input_output, solvers::solver::SatResult,
};

use super::super::solver::SatSolver;
use std::{
    collections::HashMap,
    io::{Read, Write},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use timeout_readwrite::TimeoutReader;

#[derive(Clone)]
pub struct Kissat {
    pid: Arc<Mutex<usize>>,
    stats: HashMap<String, f64>,
}

impl Kissat {
    pub fn new() -> Kissat {
        return Kissat {
            pid: Arc::new(Mutex::new(0)),
            stats: HashMap::new(),
        };
    }
}

impl SatSolver for Kissat {
    fn solve(&mut self, clauses: Clauses, num_vars: usize, timeout: &Timeout) -> SatResult {
        if timeout.time_finished() {
            return SatResult::timeout();
        }

        let string_gen_time_key: String = "string_gen_time".to_owned();
        let io_time_key = "formula_write_time".to_owned();
        let solve_time_key = "solve_time".to_owned();
        let solution_read_time_key = "solution_read_time".to_owned();

        let mut start_lock = self.pid.lock().unwrap();
        let mut child: std::process::Child = Command::new("./kissat")
            .arg("-q")
                        .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        *start_lock = child.id() as usize;
        drop(start_lock);

        let string_gen_time = Instant::now();
        let formula = input_output::to_dimacs::to_dimacs(clauses, num_vars, timeout);
        self.stats.insert(
            string_gen_time_key.clone(),
            string_gen_time.elapsed().as_secs_f64(),
        );

        let mut start_lock = self.pid.lock().unwrap();
        if *start_lock == 0 {
            return SatResult::timeout();
        }
        let a = child.stdout.take();

        if formula.is_none() {
            child.kill().unwrap();
            child.wait().unwrap();
            *start_lock = 0;
            return SatResult::timeout();
        }

        let io_time = Instant::now();
        {
            let formula = formula.unwrap();
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(formula.as_bytes()).unwrap();
            stdin.flush().unwrap();
        }
        self.stats
            .insert(io_time_key, io_time.elapsed().as_secs_f64());

        let time_remaining = timeout.remaining_time();
        if time_remaining <= 0.0
            || time_remaining.is_nan()
            || time_remaining.is_infinite()
            || timeout.time_finished()
        {
            child.kill().unwrap();
            child.wait().unwrap();
            *start_lock = 0;
            return SatResult::timeout();
        }
        let mut reader = TimeoutReader::new(a.unwrap(), Duration::from_secs_f64(time_remaining));
        drop(start_lock);

        let solve_time = Instant::now();
        let mut out = String::new();
        let res: Result<usize, std::io::Error> = reader.read_to_string(&mut out);
        let mut solver_lock = self.pid.lock().unwrap();
        *solver_lock = 0;
        self.stats
            .insert(solve_time_key, solve_time.elapsed().as_secs_f64());

        child.kill().unwrap();
        let child_exit = child.wait();
        if res.is_err() || child_exit.is_err() {
            return SatResult::timeout();
        }

        let solution_read_time = Instant::now();
        let mut solution: Vec<i32> = vec![];
        for var in out.split(&[' ', '\n'][..]) {
            let number: Result<i32, std::num::ParseIntError> = var.parse::<i32>();
            match number {
                Ok(ok) => solution.push(ok),
                Err(_) => {}
            }
        }
        self.stats.insert(
            solution_read_time_key,
            solution_read_time.elapsed().as_secs_f64(),
        );

        return if solution.len() == 0 {
            if !out.starts_with("s U") {
                return SatResult::timeout();
            }
            return SatResult::unsat();
        } else {
            SatResult::sat(solution)
        };
    }

    fn get_pid(&self) -> Arc<Mutex<usize>> {
        return self.pid.clone();
    }

    fn get_stats(&self) -> std::collections::HashMap<String, f64> {
        return self.stats.clone();
    }
}
