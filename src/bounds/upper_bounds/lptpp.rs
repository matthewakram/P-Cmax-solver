use std::time::Instant;

use crate::{
    common::common,
    problem_instance::{problem_instance::ProblemInstance, solution::Solution}, bounds::bound::Bound,
};


pub struct Lptpp {}

fn subset_sum(elements: &Vec<usize>, goal: usize) -> Vec<usize> {
    let mut dp: Vec<i16> = vec![-1; goal + 1];

    for element in 0..elements.len() {
        if dp[elements[element]] == -1 {
            dp[elements[element]] = element as i16;
        }
        for pos in 1..(dp.len() - elements[element]) {
            if dp[pos] != -1 && dp[pos] != element as i16 {
                if dp[elements[element] + pos] == -1 {
                    dp[elements[element] + pos] = element as i16;
                }
            }
        }
        if dp[goal] != -1 {
            break;
        }
    }

    let mut pointer: usize = goal;

    while dp[pointer] == -1 {
        assert_ne!(pointer, 0);
        pointer -= 1;
    }

    let mut result: Vec<usize> = vec![];
    while pointer != 0 {
        let element = dp[pointer];
        assert_ne!(element, -1);
        result.push(element as usize);
        pointer -= elements[element as usize];
    }
    return result;
}

fn is_feasable(instance: &ProblemInstance, max_makespan: usize) -> Option<Solution> {
    let mut assignment: Vec<usize> = vec![usize::MAX; instance.num_jobs];

    let mut elements = instance.job_sizes.clone();
    for processor in 0..instance.num_processors {
        let feasable = subset_sum(&elements, max_makespan);

        for job in feasable {
            elements[job] = 0;
            assignment[job] = processor;
        }
    }
    if assignment.contains(&usize::MAX) {
        return None;
    }

    let makespan: usize = common::calc_makespan(instance, &assignment);

    if makespan > max_makespan {
        return None;
    }

    return Some(Solution {
        makespan,
        assignment,
    });
}


impl Bound for Lptpp{
    fn bound(&self, problem: &ProblemInstance, lower_bound: usize, upper_bound: Option<Solution>, timeout: f64) -> (usize, Option<Solution>) {
        let mut makespan_to_check = lower_bound;
        let current_bound = upper_bound.as_ref().unwrap();
        let mut remaining_time = timeout;
        loop {
            if makespan_to_check >= current_bound.makespan || remaining_time <= 0.0 {
                return (lower_bound, upper_bound);
            }

            let start_time = Instant::now();
            
            let sol = is_feasable(problem, makespan_to_check);
            if sol.is_some() {
                return (lower_bound, sol);
            }

            remaining_time -= start_time.elapsed().as_secs_f64();
            makespan_to_check += 1;
        }
    }
}