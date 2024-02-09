use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use rand::{rngs::ThreadRng, Rng};

#[derive(Clone)]
pub struct GurobiDecider {}

impl GurobiDecider {
    pub fn new() -> GurobiDecider {
        return GurobiDecider {};
    }
}

impl GurobiDecider {

    pub fn solve(
        &mut self,
        formula: String,
        timeout: &crate::common::timeout::Timeout,
    ) -> Option<HashMap<String, usize>> {



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


        let child = Command::new("gurobi_cl")
            .arg("Threads=1")
            .arg("IntegralityFocus=1")
            .arg("MemLimit=7")
            .arg("SolutionLimit=1")
            .arg(format!("ResultFile={}.sol", file_name))
            .arg(format!("TimeLimit={}", time_remaining))
            .arg(format!("{}", file_name))
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        let out = String::from(std::str::from_utf8(&child.stdout).unwrap());

        if out.contains("Time limit reached") || out.contains("Error 10001: Out of memory") {
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

        let _ = fs::remove_file(&file_name).unwrap();
        let _ = fs::remove_file(format!("{}.sol", file_name));
        if r.is_ok() {
            let num_bytes_read = r.unwrap();

            if num_bytes_read < 2 {
                return None;
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
            .map(|(key, val)| (key.to_string(), val.parse::<usize>().unwrap_or(usize::MAX)))
            .collect::<HashMap<String, usize>>();

        if assignments.iter().any(|(_, x)| *x == usize::MAX) {
            println!("could not find integral solution\n");
            return None;
        }

        return Some(assignments);
    }
}
