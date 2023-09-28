use super::lower_bound::LowerBound;


pub struct MaxJobSize{
}

impl LowerBound for MaxJobSize {
    fn get_lower_bound(&self, problem: &crate::problem_instance::problem_instance::ProblemInstance) -> usize {
        return *problem.job_sizes.iter().max().unwrap();
    }
}