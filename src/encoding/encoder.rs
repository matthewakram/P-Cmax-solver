use std::fmt::Debug;

use crate::{problem_instance::{partial_solution::PartialSolution, solution::Solution, problem_instance::ProblemInstance}, common::timeout::Timeout};

use super::problem_encoding::one_hot_encoding::OneHot;
use dyn_clone::DynClone;

pub trait OneHotEncoder : Encoder + OneHot + DynClone {}

dyn_clone::clone_trait_object!(OneHotEncoder);


pub trait Encoder : DynClone + Send {
    fn basic_encode(&mut self, partial_solution: &PartialSolution, makespan: usize, timeout: &Timeout, max_num_clauses: usize) -> bool;
    fn output(&mut self) -> Clauses;
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
pub struct Clauses {
    num_clauses: usize,
    clauses : Vec<i32>
}


impl Clauses {
    pub fn new() -> Clauses {
        return Clauses {num_clauses: 0,  clauses: vec![] }
    }
    pub fn add_clause(&mut self, mut clause: Clause) {
        self.clauses.append(&mut clause.vars);
        self.clauses.push(0);
        self.num_clauses += 1;
    }

    pub fn add_many_clauses(&mut self, clauses: &mut Clauses) {
        self.num_clauses += clauses.num_clauses;
        self.clauses.append(&mut clauses.clauses);
    }

    pub fn len(&self) -> usize {
        return self.clauses.len();
    } 

    pub fn iter(self: &Self) -> impl Iterator<Item=&i32>{
        self.clauses.iter()
  }

    pub fn get_num_clauses(&self) -> usize {
        return self.num_clauses;
    }
}

pub struct Clause{
    pub vars: Vec<i32>,
}

impl Debug for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{:?}", self.vars);
    }
}


