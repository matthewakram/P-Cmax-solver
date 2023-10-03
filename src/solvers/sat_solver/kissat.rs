use super::super::solver::SatSolver;
use std::process::Command;


pub struct Kissat{

}

impl SatSolver for Kissat {
    fn solve(&self, file_name: &str) -> Option<Vec<i32>> {
        let result = Command::new("./kissat")
        .arg(file_name)
        .arg("-q")
        .output()
        .expect("./kissat command failed to start");

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