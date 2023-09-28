use crate::problem_instance::solution::Solution;

use super::upper_bound::InitialUpperBound;
use super::super::super::problem_instance::problem_instance::ProblemInstance;

/// implements the lpt heuristic for upper and lower bounds

pub struct LPT{
    
}


fn index_of_min(list: &Vec<usize>) -> usize{
    let (index, _) = list.iter().enumerate().min_by_key(|(_, x)| *x).unwrap();
    return index;
}

impl InitialUpperBound for LPT{
    fn get_upper_bound(& self, instance: &ProblemInstance) -> Solution{
        let mut solution: Vec<usize> = vec![];
        let mut total_sizes: Vec<usize> = vec![0; instance.num_processors];

        for i in 0..instance.job_sizes.len() {
            let emptiest_processor = index_of_min(&total_sizes);
            total_sizes[emptiest_processor] += instance.job_sizes[i];
            solution.push(emptiest_processor);
        }

        let makespan = *total_sizes.iter().max().unwrap();

        return Solution{
            makespan,
            assignment: solution
        }
    }
}


