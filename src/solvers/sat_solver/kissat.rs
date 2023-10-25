use wait_timeout::ChildExt;

use super::super::solver::SatSolver;
use std::{process::{Command, Stdio}, time::Duration};


pub struct Kissat{

}

impl SatSolver for Kissat {
    fn solve(&self, file_name: &str, timeout: u64) -> Option<Vec<i32>> {
        //let result = Command::new("./kissat")
        //.arg(file_name)
        //.arg("-q")
        //.output()
        //.expect("./kissat command failed to start");

        let mut child = Command::new("./kissat").arg(file_name).arg("-q").stdout(Stdio::piped()).spawn().unwrap();

        let one_sec = Duration::from_secs(timeout);
        let status =  child.wait_timeout(one_sec).unwrap() ;
        if status.is_none(){
            child.kill().unwrap();
            return None;
        }
        
        let result = child.wait_with_output().expect("./kissat command failed to start");

        let output = std::str::from_utf8(&result.stdout).unwrap();

        let mut solution: Vec<i32> = vec![];
        for var in  output.split(&[' ', '\n'][..]){
            let number = var.parse::<i32>();
            match number {
                Ok(ok) => solution.push(ok),
                Err(_) => {},
            }
        }
        
        return if solution.len() == 0 {None} else {Some(solution)};
    }
}