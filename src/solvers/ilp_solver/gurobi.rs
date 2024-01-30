use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use rand::{rngs::ThreadRng, Rng};

use crate::{
    encoding::ilp_encoder::ILPEncoder,
    problem_instance::partial_solution::PartialSolution,
    problem_simplification::{
        fill_up_rule::FillUpRule, final_simp_rule::FinalizeRule, half_size_rule::HalfSizeRule,
        simplification_rule::SimpRule,
    },
    solvers::solver_manager::SolverManager,
};

#[derive(Clone)]
pub struct Gurobi {
    encoder: Box<dyn ILPEncoder>,
}

impl Gurobi {
    pub fn new(encoder: Box<dyn ILPEncoder>) -> Gurobi {
        return Gurobi { encoder };
    }
}

impl SolverManager for Gurobi {
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

        let success = self.encoder
            .encode(&partial_solution, lower, makespan_to_test, timeout);
        if !success {
            return None;
        }
        let formula = self.encoder.get_encoding();
        let mut rng: ThreadRng = ThreadRng::default();
        let mut i: usize = rng.gen();
        let mut file_name = format!("./ILP_formula{}.lp", i);
        while Path::new(&file_name).is_file() {
            i = rng.gen();
            file_name = format!("./ILP_formula{}.lp", i);
        }
        let mut file = File::create(Path::new(&file_name)).expect("unable to create model file");
        file.write(formula.as_bytes())
            .expect("could not write model to file");
        drop(formula);

        let time_remaining = timeout.remaining_time();
        if time_remaining <= 0.0
            || time_remaining.is_nan()
            || time_remaining.is_infinite()
            || timeout.time_finished()
        {
            let _ = fs::remove_file(&file_name);
            return None;
        }

        let child  = Command::new("gurobi_cl")
            .arg("Threads=1")
            .arg("Threads=1")
            .arg("MemLimit=7")
            .arg(format!("ResultFile={}.sol", file_name))
            .arg(format!("TimeLimit={}", time_remaining))
            .arg(format!("{}", file_name))
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        let out = String::from(std::str::from_utf8(&child.stdout).unwrap());

        if out.contains("Time limit reached") || out.contains("Error 10001: Out of memory"){
            let _ = fs::remove_file(&file_name).unwrap();
            let _ = fs::remove_file(format!("{}.sol", file_name));    
            return None;
        }

        let f = File::open(format!("{}.sol", file_name));
        if f.is_err() {
            let _ = fs::remove_file(&file_name).unwrap();
            let _ = fs::remove_file(format!("{}.sol", file_name));
            return None;
        }
        let mut data = vec![];
        let mut f = f.unwrap();
        let r = f.read_to_end(&mut data);
        
        // TODO: move the remove file back here
        let _ = fs::remove_file(&file_name).unwrap();
        let _ = fs::remove_file(format!("{}.sol", file_name));
        if r.is_ok() {
            let num_bytes_read = r.unwrap();
            
            if num_bytes_read < 2 {
                return Some(upper.clone());
            }
        }
        
        let solution = std::str::from_utf8(&data).unwrap();
        let lines: Vec<&str> = solution.split("\n").collect();
        let assignments = lines
        .iter()
        .filter(|x| !x.starts_with('#'))
        .map(|x| x.split(" ").collect::<Vec<&str>>())
        .filter(|x| x.len() > 1)
        .map(|x| (x[0], x[1]))
        .map(|(key, val)| (key.to_string(), val.parse::<usize>().unwrap()))
        .collect::<HashMap<String, usize>>();
    
        return Some(self.encoder.decode(instance, assignments));
    }
}
