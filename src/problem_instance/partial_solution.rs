use super::problem_instance::ProblemInstance;


#[derive(Clone, Debug)]
pub struct PartialSolution{
    pub instance: ProblemInstance,
    /// The procs you can assign this job to. i.e. possible_allocations[job] contains possible allocations
    pub possible_allocations: Vec<Vec<usize>>,
    pub assigned_makespan: Vec<usize>
}

impl PartialSolution {
    pub fn new(instance: ProblemInstance) -> PartialSolution {
        let mut possible_allocations: Vec<Vec<usize>> = vec![];

        let mut assigned_makespan: Vec<usize> = vec![0;instance.num_processors];
        
        for job in 0..instance.num_jobs {
            possible_allocations.push(vec![]);
            for process in 0..instance.num_processors.min(job +1){
                possible_allocations[job].push(process);
            }
        }

        assigned_makespan[0] = instance.job_sizes[0];

        return PartialSolution { instance, possible_allocations, assigned_makespan};
    }
}