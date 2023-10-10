use crate::problem_instance::{partial_solution::PartialSolution, solution::Solution, problem_instance::ProblemInstance};

pub trait Encoder {
    fn basic_encode(&mut self, partial_solution: &PartialSolution, makespan: usize);
    fn output(&self) -> Vec<Clause>;
    fn decode(&self, instance: &ProblemInstance, solution: &Vec<i32>) -> Solution;
    fn get_num_vars(&self) -> usize;
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

    pub fn peek(&self) -> usize {
        return *(self.current.clone().peekable().peek().unwrap());
    }
    pub fn jump_to(&mut self, to: usize) {
        self.current = std::ops::RangeFrom { start: (to) };
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Clause{
    pub vars: Vec<i32>,
}


//TODO encoder goals:
// two elements of the same size: one has to come after the other one
// fill rule
// replacement rule