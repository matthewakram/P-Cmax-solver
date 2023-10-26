
pub trait SatSolver {
    fn solve(&self, file_name: &str, timeout: f64) -> SatResult;
}

pub struct SatResult {
    timeout: bool,
    sat: bool,
    solution: Option<Vec<i32>>,
}

impl SatResult {
    pub fn timeout() -> SatResult{
        return SatResult{
            timeout: true,
            sat: false,
            solution : None,
        }
    }
    pub fn sat(result: Vec<i32>) -> SatResult{
        return SatResult{
            timeout: false,
            sat: true,
            solution : Some(result),
        }
    }

    pub fn unsat() -> SatResult{
        return SatResult{
            timeout: false,
            sat: false,
            solution : None,
        }
    }

    pub fn unwrap(self) -> Option<Vec<i32>> {
        if self.timeout {
            panic!("unwrapped a timeout");
        }

        return self.solution;
    }

    pub fn is_sat(&self)-> bool{
        return self.sat;
    }

    pub fn is_timeout(&self) -> bool{
        return self.timeout;
    }
}