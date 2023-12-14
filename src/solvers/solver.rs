use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use dyn_clone::DynClone;

use crate::{common::timeout::Timeout, encoding::encoder::Clauses};

/// A SatSolver is a wrapper for SAT based solving, (whichever that may be). It is responsible for starting and communicating with the SAT solver.
pub trait SatSolver: DynClone + Send {
    /// Since SATSolvers might often be other programs that must be started as a different process, the get_pid command returns a refrence to the
    /// pid of that process (if existant). When the extra process is running, the returned variable contains the pid of the process, and 0 otherwise.
    /// This method is avaiable in order to force the stoping of computation from a different thread.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let solver = Kissat::new();
    /// let pid = solver.get_pid();
    /// ...
    /// // In a different thread
    /// let mut pid = pid.lock().unwrap();
    /// if *pid != 0 {
    ///     ...
    ///     // kill process
    ///     ...
    ///     *sat_pid = 0;
    /// }
    /// drop(pid);
    /// ```
    fn get_pid(&self) -> Arc<Mutex<usize>>;

    /// A method that solved the given formula or exists if the given timout is runs out.
    /// Note that in the case that the timeout is interupted, tbis method might not stop immediately.
    /// In order to force stop solving, refer to the get_pid() method.
    fn solve(&mut self, clauses: Clauses, num_vars: usize, timeout: &Timeout) -> SatResult;

    /// Gets statistics about the solving e.g. solve time, write time, read time ...
    fn get_stats(&self) -> HashMap<String, f64>;
}

dyn_clone::clone_trait_object!(SatSolver);


/// A structure repsenting a Result from the sat solver.
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
