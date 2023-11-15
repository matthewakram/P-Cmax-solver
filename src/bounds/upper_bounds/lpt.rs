use crate::bounds::bound::Bound;
use crate::common::timeout::Timeout;
use crate::problem_instance::solution::Solution;

use super::super::super::problem_instance::problem_instance::ProblemInstance;

/// implements the lpt heuristic for upper and lower bounds

pub struct LPT{
    
}


fn index_of_min(list: &Vec<usize>) -> usize{
    let (index, _) = list.iter().enumerate().min_by_key(|(_, x)| *x).unwrap();
    return index;
}

impl Bound for LPT {
    fn bound(&self, instance: &ProblemInstance, lower_bound: usize, upper_bound: Option<Solution>, _timeout: &Timeout) -> (usize, Option<Solution>) {
        let mut solution: Vec<usize> = vec![];
        let mut total_sizes: Vec<usize> = vec![0; instance.num_processors];

        for i in 0..instance.job_sizes.len() {
            let emptiest_processor = index_of_min(&total_sizes);
            total_sizes[emptiest_processor] += instance.job_sizes[i];
            solution.push(emptiest_processor);
        }

        let makespan = *total_sizes.iter().max().unwrap();
        //TODO: this can be improved from the lit, but I CBA rn
        let new_lower_bound = (makespan * 3) / 4 + 1;
        let lower_bound = new_lower_bound.max(lower_bound);

        if upper_bound.is_none() || upper_bound.as_ref().unwrap().makespan > makespan {
            return (lower_bound, Some( Solution{
                makespan,
                assignment: solution
            }))
        }
        return (lower_bound, upper_bound)
    }
}