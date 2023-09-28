use crate::problem_instance::problem_instance::ProblemInstance;




pub trait LowerBound {
    fn get_lower_bound(&self, problem: &ProblemInstance) -> usize;
}