use super::solution::Solution;

#[derive(Clone, Debug)]
pub struct ProblemInstance {
    /// The basic instance of the PCMAX problem. Job Sizes is sorted in descending order.
    /// Since we order our jobs, the job order stores the order with which we can get the original jobs order back
    pub num_processors: usize,
    pub num_jobs: usize,
    pub job_sizes: Vec<usize>,
    job_order: Vec<usize>,
}

impl ProblemInstance {
    pub fn new(num_processors: usize, num_jobs: usize, job_sizes: Vec<usize>) -> ProblemInstance {
        assert!(job_sizes.len() == num_jobs);
        let mut unordered_jobs: Vec<(usize, usize)> =
            job_sizes.iter().enumerate().map(|(i, x)| (i, *x)).collect();
        unordered_jobs.sort_by(|(_, xx), (_, yy)| (*yy).cmp(xx));

        let job_order = unordered_jobs.iter().map(|(i, _)| *i).collect();
        let job_sizes = unordered_jobs.iter().map(|(_, x)| *x).collect();

        ProblemInstance {
            num_processors,
            num_jobs,
            job_sizes,
            job_order,
        }
    }

    pub fn finalize_solution(&self, sol: Solution) -> Solution {
        let mut final_assignment: Vec<usize> = vec![0; self.num_jobs];
        for i in 0..self.num_jobs {
            final_assignment[self.job_order[i]] = sol.assignment[i];
        }

        return Solution {
            makespan: sol.makespan,
            assignment: final_assignment,
        };
    }
}
