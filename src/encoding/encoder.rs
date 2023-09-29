use crate::problem_instance::{partial_solution::PartialSolution, solution::Solution};

pub trait Encoder {
    fn basic_encode(&mut self, partial_solution: &PartialSolution);
    fn output(&self);
    fn decode(&self) -> Solution;
}

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
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Clause{
    pub vars: Vec<i32>,
}

