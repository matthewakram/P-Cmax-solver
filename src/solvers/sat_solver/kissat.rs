use crate::solvers::solver::SatResult;

use super::super::solver::SatSolver;
use std::{process::{Command, Stdio}, time::Duration, io::Read};
use timeout_readwrite::TimeoutReader;


pub struct Kissat{

}

impl SatSolver for Kissat {
    fn solve(&self, file_name: &str, timeout: f64) -> SatResult {

        let child = Command::new("./kissat").arg(file_name).arg("-q").stdout(Stdio::piped()).spawn().unwrap();


        let mut reader = TimeoutReader::new(child.stdout.unwrap(), Duration::from_secs_f64(timeout));
        let mut out = String::new();
        

        let res: Result<usize, std::io::Error> = reader.read_to_string(&mut out);
        
        if res.is_err() {
            return SatResult::timeout();
        }
                

        let mut solution: Vec<i32> = vec![];
        for var in  out.split(&[' ', '\n'][..]){
            let number = var.parse::<i32>();
            match number {
                Ok(ok) => solution.push(ok),
                Err(_) => {},
            }
        }
        
        return if solution.len() == 0 {SatResult::unsat()} else {SatResult::sat(solution)};
    }
}