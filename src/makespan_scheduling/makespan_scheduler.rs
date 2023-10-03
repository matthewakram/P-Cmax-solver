use crate::problem_instance::{problem_instance::ProblemInstance, solution::Solution};


pub trait MakespanScheduler {
    fn next_makespan(&mut self, instance: &ProblemInstance, solution: &Solution, lower: usize, upper: usize) -> usize;
}