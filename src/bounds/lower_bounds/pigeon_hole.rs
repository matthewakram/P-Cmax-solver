use crate::bounds::bound::Bound;




pub struct PigeonHole{}



impl Bound for PigeonHole {
    fn bound(&self, problem: &crate::problem_instance::problem_instance::ProblemInstance, lower_bound: usize, upper_bound: Option<crate::problem_instance::solution::Solution>, _timeout: f64) -> (usize, Option<crate::problem_instance::solution::Solution>) {
        let sum: usize = problem.job_sizes.iter().sum();
        let mut bound = sum / problem.num_processors;
        if sum % problem.num_processors != 0 {
            bound += 1;
        }
        return (lower_bound.max(bound), upper_bound)
    }
}