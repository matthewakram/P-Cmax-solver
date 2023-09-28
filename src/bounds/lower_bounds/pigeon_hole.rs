use super::lower_bound::LowerBound;



pub struct PigeonHole{}


impl LowerBound for PigeonHole {
    fn get_lower_bound(&self, problem: &crate::problem_instance::problem_instance::ProblemInstance) -> usize {
        let sum: usize = problem.job_sizes.iter().sum();
        let mut bound = sum / problem.num_processors;
        if sum % problem.num_processors != 0 {
            bound += 1;
        }
        return bound;
    }
}