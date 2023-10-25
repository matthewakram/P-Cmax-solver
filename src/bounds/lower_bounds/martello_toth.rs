use crate::{problem_instance::{problem_instance::ProblemInstance, self}, bounds::bound::Bound};



// This is USELESS

pub struct MartelloToth{
}

pub fn get_j1(instance: &ProblemInstance, makespan_to_test: usize, p: usize) -> Vec<usize>{
    return instance.job_sizes.iter().filter(|pj| **pj <= makespan_to_test/2 && makespan_to_test-p < **pj).map(|x| *x).collect();
}

pub fn get_j2(instance: &ProblemInstance, makespan_to_test: usize, p: usize) -> Vec<usize> {
    return instance.job_sizes.iter().filter(|pj| makespan_to_test/2 < **pj && **pj <= makespan_to_test - p).map(|x| *x).collect();
}

pub fn get_j3(instance: &ProblemInstance, makespan_to_test: usize, p: usize) -> Vec<usize> {
    return instance.job_sizes.iter().filter(|pj| p <= **pj && **pj <= makespan_to_test/2).map(|x| *x).collect();
}

pub fn inner_sum(j1: Vec<usize>, j2: Vec<usize>, j3: Vec<usize>, C: usize)-> usize{
    let first_sum: i64 = j3.iter().sum::<usize>() as i64;
    let middle : i64 = (j2.len() * C) as i64;
    let second_sum: i64 = j2.iter().sum::<usize>() as i64;
    let out = first_sum - middle + second_sum;
    if out < 0 {
        return 0
    }
    let out = out as usize;

    let out = if out % C == 0 {out / C} else {(out/C) + 1};
     
    return out;
}

pub fn bin_packing_bound(instance: &ProblemInstance, makespan_to_test: usize) -> usize {
    let mut best_bound = 0;
    for i in 0..instance.num_jobs{
        if i < instance.num_jobs - 1 && instance.job_sizes[i] == instance.job_sizes[i+1] {
            continue;
        } 
        let j1 = get_j1(instance, makespan_to_test, i);
        let j2 = get_j2(instance, makespan_to_test, i);
        let j3 = get_j3(instance, makespan_to_test, i);
        let bound = j1.len() + j2.len() + inner_sum(j1, j2, j3, makespan_to_test);
        best_bound = best_bound.max(bound);
    }
    return best_bound
}

impl Bound for MartelloToth {
    fn bound(&self, problem: &ProblemInstance, lower_bound: usize, upper_bound: Option<problem_instance::solution::Solution>) -> (usize, Option<problem_instance::solution::Solution>) {
        let mut makespan_to_check = lower_bound;
        let current_bound = upper_bound.as_ref().unwrap();
        loop {
            if makespan_to_check >= current_bound.makespan {
                return (current_bound.makespan, upper_bound);
            }
            
            let min_num_needed_procs = bin_packing_bound(problem, makespan_to_check);
            if min_num_needed_procs <= problem.num_processors {
                return (makespan_to_check, upper_bound);
            }
            makespan_to_check += 1;
        }
    }
}