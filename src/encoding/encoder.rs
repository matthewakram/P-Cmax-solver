use std::fmt::Debug;

use crate::{problem_instance::{partial_solution::PartialSolution, solution::Solution, problem_instance::ProblemInstance}, common::timeout::Timeout};

use super::problem_encoding::one_hot_encoding::OneHot;
use dyn_clone::DynClone;

pub trait OneHotEncoder : Encoder + OneHot + DynClone {}

dyn_clone::clone_trait_object!(OneHotEncoder);


pub trait Encoder : DynClone + Send {
    fn basic_encode(&mut self, partial_solution: &PartialSolution, makespan: usize, timeout: &Timeout) -> bool;
    fn output(&self) -> Vec<Clause>;
    fn decode(&self, instance: &ProblemInstance, solution: &Vec<i32>) -> Solution;
    fn get_num_vars(&self) -> usize;
}

dyn_clone::clone_trait_object!(Encoder);


#[derive(Clone)]
pub struct VarNameGenerator {
    current: std::ops::RangeFrom<usize>,
}

impl VarNameGenerator {
    pub fn new() -> VarNameGenerator {
        return VarNameGenerator {
            current: std::ops::RangeFrom { start: (1) },
        };
    }
    pub fn next(&mut self) -> usize {
        return self.current.next().unwrap();
    }

    pub fn peek(&self) -> usize {
        return *(self.current.clone().peekable().peek().unwrap());
    }
    pub fn jump_to(&mut self, to: usize) {
        self.current = std::ops::RangeFrom { start: (to) };
    }
}

#[derive( Eq, PartialEq, Clone)]
pub struct Clause{
    pub vars: Vec<i32>,
}

impl Debug for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{:?}", self.vars);
    }
}


