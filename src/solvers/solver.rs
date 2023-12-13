use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use dyn_clone::DynClone;

use crate::{common::timeout::Timeout, encoding::encoder::Clauses};

pub trait SatSolver: DynClone + Send {
    fn get_pid(&self) -> Arc<Mutex<usize>>;
    fn solve(&mut self, clauses: Clauses, num_vars: usize, timeout: &Timeout) -> SatResult;
    fn get_stats(&self) -> HashMap<String, f64>;
}

dyn_clone::clone_trait_object!(SatSolver);

pub struct SatResult {
    timeout: bool,
    sat: bool,
    solution: Option<Vec<i32>>,
}

impl SatResult {
    pub fn timeout() -> SatResult {
        return SatResult {
            timeout: true,
            sat: false,
            solution: None,
        };
    }
    pub fn sat(result: Vec<i32>) -> SatResult {
        return SatResult {
            timeout: false,
            sat: true,
            solution: Some(result),
        };
    }

    pub fn unsat() -> SatResult {
        return SatResult {
            timeout: false,
            sat: false,
            solution: None,
        };
    }

    pub fn unwrap(self) -> Option<Vec<i32>> {
        if self.timeout {
            panic!("unwrapped a timeout");
        }

        return self.solution;
    }

    pub fn is_sat(&self) -> bool {
        return self.sat;
    }

    pub fn is_timeout(&self) -> bool {
        return self.timeout;
    }

    pub fn is_unsat(&self) -> bool {
        return !self.is_timeout() && !self.is_sat();
    }
}
