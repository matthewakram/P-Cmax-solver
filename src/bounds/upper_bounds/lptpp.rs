use crate::{problem_instance::{problem_instance::ProblemInstance, solution::Solution}, common::common};

use super::upper_bound::InitialUpperBound;


pub struct Lptpp {
    pub lower_bound: usize
}

fn subset_sum(elements: &Vec<usize>, goal: usize) -> Option<Vec<usize>>{
    let mut dp: Vec<i32> = vec![-1; goal+1];

    for element in 0..elements.len(){
        if dp[elements[element]] == -1{
            dp[elements[element]] = element as i32;
        } 
        for pos in 1..(dp.len()-elements[element]) {
            if dp[pos] != -1 && dp[pos] != element as i32{
                if dp[elements[element] + pos] == -1{
                    dp[elements[element] + pos] = element as i32;
                }
            }
        }
        if dp[goal] != -1{
            break;
        }
    }

    let mut pointer: usize = goal;

    while dp[pointer] == -1 {
        assert_ne!(pointer, 0);
        pointer -= 1;
    }

    let mut result:Vec<usize> = vec![];
    while pointer != 0 {
        let element = dp[pointer];
        assert_ne!(element, -1);
        result.push(element as usize);
        pointer -= elements[element as usize];
    }
    return Some(result);
}

fn is_feasable(instance: &ProblemInstance, max_makespan: usize) -> Option<Solution> {
    let mut assignment: Vec<usize> = vec![usize::MAX;instance.num_jobs];

    let mut elements = instance.job_sizes.clone();
    for processor in 0..instance.num_processors {
        let feasable = subset_sum(&elements, max_makespan);
        if feasable.is_none() {
            break;
        }
        let feasable = feasable.unwrap();
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

    return Some(Solution{makespan, assignment});
}

impl InitialUpperBound for Lptpp {
    fn get_upper_bound(&self, instance: &ProblemInstance) -> Solution {
        let mut makespan_to_check = self.lower_bound;
        loop {
            let sol = is_feasable(instance, makespan_to_check);
            if sol.is_some() {
                return sol.unwrap();
            }
            makespan_to_check += 1;
        }
    }
}