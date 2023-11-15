
use crate::{problem_instance::{problem_instance::ProblemInstance, solution::Solution}, bounds::bound::Bound, common::timeout::Timeout};


pub struct Lptp {}


fn is_feasable(instance: &ProblemInstance, max_makespan: usize) -> Option<Solution> {
    let mut assignment: Vec<usize> = vec![usize::MAX; instance.num_jobs];
    let mut total_sizes: Vec<usize> = vec![0; instance.num_processors];
    let mut available_sizes = instance.job_sizes.clone();

    for i in 0..instance.job_sizes.len() {
        if assignment[i] != usize::MAX {
            continue;
        }
        let emptiest_processor: usize = total_sizes.iter().enumerate().min_by_key(|(_,x)| *x).map(|(i,_)| i).unwrap();
        total_sizes[emptiest_processor] += instance.job_sizes[i];
        assignment[i] = emptiest_processor;
        available_sizes[i] = 0;

        if total_sizes[emptiest_processor] > max_makespan {
            return None;
        }

        if total_sizes[emptiest_processor] == max_makespan{
            continue;
        }
        
        
        let index_of_fur_element: Option<usize> = available_sizes[i+1..].iter().position(|x| x == &(max_makespan - total_sizes[emptiest_processor]));
        if index_of_fur_element.is_some() {
            let index = index_of_fur_element.unwrap() + i + 1;
            total_sizes[emptiest_processor] += instance.job_sizes[index];
            assignment[index] = emptiest_processor;
            available_sizes[index] = 0;
        }
    }
    
    let makespan = *total_sizes.iter().max().unwrap();

    return Some( Solution{
        makespan,
        assignment: assignment
    })
}


impl Bound for Lptp{
    fn bound(&self, problem: &ProblemInstance, lower_bound: usize, upper_bound: Option<Solution>, timeout: &Timeout) -> (usize, Option<Solution>) {
        let mut makespan_to_check = lower_bound;
        let current_bound = upper_bound.as_ref().unwrap();

        loop {
            if makespan_to_check >= current_bound.makespan || timeout.time_finished() {
                return (lower_bound, upper_bound);
            }
            
            let sol = is_feasable(problem, makespan_to_check);
            if sol.is_some() {
                return (lower_bound, sol);
            }

            makespan_to_check += 1;
        }
    }
}