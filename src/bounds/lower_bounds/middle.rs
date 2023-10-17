use super::lower_bound::LowerBound;


pub struct MiddleJobs{
}

impl LowerBound for MiddleJobs {
    fn get_lower_bound(&self, problem: &crate::problem_instance::problem_instance::ProblemInstance) -> usize {
        assert!(problem.num_processors < problem.num_jobs);
        return problem.job_sizes[problem.num_processors - 1] + problem.job_sizes[problem.num_processors];
    }
}