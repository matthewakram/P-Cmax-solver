use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use rand::{rngs::ThreadRng, Rng};
use timeout_readwrite::TimeoutReader;

use crate::{
    encoding::cplex_model_encoder::CPLEXModelEncoder,
    problem_instance::partial_solution::PartialSolution,
    problem_simplification::{
        fill_up_rule::FillUpRule, final_simp_rule::FinalizeRule, half_size_rule::HalfSizeRule,
        simplification_rule::SimpRule,
    },
    solvers::solver_manager::SolverManager,
};

#[derive(Clone)]
pub struct CPELXSolver {
    encoder: Box<dyn CPLEXModelEncoder>,
}

impl CPELXSolver {
    pub fn new(encoder: Box<dyn CPLEXModelEncoder>) -> CPELXSolver {
        return CPELXSolver { encoder };
    }
}

impl SolverManager for CPELXSolver {
    fn solve(
        &mut self,
        instance: &crate::problem_instance::problem_instance::ProblemInstance,
        lower: usize,
        upper: &crate::problem_instance::solution::Solution,
        timeout: &crate::common::timeout::Timeout,
        _verbose: bool,
    ) -> Option<crate::problem_instance::solution::Solution> {
        let makespan_to_test = upper.makespan - 1;
        let partial_solution = PartialSolution::new(instance.clone());

        let mut hsr = HalfSizeRule {};
        let mut fur: FillUpRule = FillUpRule {};
        let mut finalize: FinalizeRule = FinalizeRule {};
        let partial_solution: PartialSolution =
            hsr.simplify(&partial_solution, makespan_to_test).unwrap();
        let partial_solution: PartialSolution =
            fur.simplify(&partial_solution, makespan_to_test).unwrap();
        let partial_solution = finalize.simplify(&partial_solution, makespan_to_test);
        if partial_solution.is_none() {
            return Some(upper.clone());
        }
        let partial_solution = partial_solution.unwrap();

        self.encoder
            .encode(&partial_solution, lower, makespan_to_test, timeout);
        let formula = self.encoder.get_encoding();
        let mut rng: ThreadRng = ThreadRng::default();
        let mut i: usize = rng.gen();
        let mut file_name = format!("./CP_formula{}.data", i);
        while Path::new(&file_name).is_file() {
            i = rng.gen();
            file_name = format!("./CP_formula{}.data", i);
        }

        let mut file = File::create(Path::new(&file_name)).expect("unable to create model file");
        file.write(formula.as_bytes())
            .expect("could not write model to file");

        let mut child: std::process::Child =
            Command::new("/Applications/CPLEX_Studio2211/opl/bin/x86-64_osx/oplrun")
                .arg("-deploy")
                .arg("-v")
                .arg(self.encoder.get_mod_file_path())
                .arg(file_name.clone())
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();

        let output = child.stdout.take();
        let time_remaining = timeout.remaining_time();
        if time_remaining <= 0.0
            || time_remaining.is_nan()
            || time_remaining.is_infinite()
            || timeout.time_finished()
        {
            child.kill().unwrap();
            child.wait().unwrap();
            let _ = fs::remove_file(&file_name);
            return None;
        }

        let mut reader =
            TimeoutReader::new(output.unwrap(), Duration::from_secs_f64(time_remaining));

        let mut out = String::new();
        let res: Result<usize, std::io::Error> = reader.read_to_string(&mut out);

        let _ = fs::remove_file(&file_name);
        child.kill().unwrap();
        child.wait().unwrap();
        if res.is_err() {
            return None;
        }

        if out.contains("OBJECTIVE: ") {
            // DONE case and managed to improve it
            return Some(self.encoder.decode(instance, out));
        } else if out.contains("<<< no solution") {
            // UNSAT CASE
            return Some(upper.clone());
        } else {
            // Timeout case
            return None;
        }
    }
}
