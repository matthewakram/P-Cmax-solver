
use rand::{Rng, rngs::ThreadRng, seq::SliceRandom};

use crate::{problem_instance::{problem_instance::ProblemInstance, solution::{Solution, self}}, bounds::{bound::Bound, upper_bounds}, common::timeout::Timeout};


pub struct MSS {}

fn subset_sum(elements: &Vec<usize>, goal: usize, lower_bound: usize) -> Vec<usize> {
    let mut dp: Vec<i16> = vec![-1; goal.max(lower_bound) + 1];

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

    let mut pointer: usize = dp.len()-1;

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

impl MSS{
    fn improve_procs(&self, instance: &ProblemInstance, first_proc: usize, second_proc: usize, assignments: &mut Vec<Vec<usize>>, assigned_makespans: &mut Vec<usize>, rng: &mut ThreadRng, lower_bound: usize) {

        if assigned_makespans[first_proc] == assigned_makespans[second_proc] 
        || assigned_makespans[first_proc] == assigned_makespans[second_proc] - 1
        || assigned_makespans[second_proc] == assigned_makespans[first_proc] - 1{
            return;
        }
        
        let mut jobs_to_assign = assignments[first_proc].clone();
        jobs_to_assign.append(&mut assignments[second_proc].clone());
        jobs_to_assign.shuffle(rng);

        let elements: Vec<usize> = jobs_to_assign.iter().map(|x| instance.job_sizes[*x]).collect();
        let new_proc_assignment: Vec<usize> = subset_sum(&elements, (assigned_makespans[first_proc] + assigned_makespans[second_proc]) / 2, lower_bound);
        let mut new_proc_assignment: Vec<usize> = new_proc_assignment.iter().map(|x| jobs_to_assign[*x]).collect();
        let mut second_proc_assignment: Vec<usize> = vec![];

        let mut first_proc_assigned_makespan = 0;
        let mut second_proc_assigned_makespan = 0;

        let mut new_proc_assignment_pointer = 0;
        new_proc_assignment.reverse();
        for i in jobs_to_assign.clone() {
            if new_proc_assignment_pointer < new_proc_assignment.len() && new_proc_assignment[new_proc_assignment_pointer] == i {
                first_proc_assigned_makespan += instance.job_sizes[new_proc_assignment[new_proc_assignment_pointer]];
                new_proc_assignment_pointer += 1;
            } else {
                second_proc_assignment.push(i);
                second_proc_assigned_makespan += instance.job_sizes[i];
            }
        }

        assignments[first_proc] = new_proc_assignment;
        assignments[second_proc] = second_proc_assignment;
        assigned_makespans[first_proc] = first_proc_assigned_makespan;
        assigned_makespans[second_proc] = second_proc_assigned_makespan;
    }
}



impl Bound for MSS {
    fn bound(&self, problem: &ProblemInstance, lower_bound: usize, upper_bound: Option<Solution>, timeout: &Timeout) -> (usize, Option<Solution>) {
        
        assert!(upper_bound.is_some());
        let upper_bound = upper_bound.unwrap();
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

        let mut proc_assignments: Vec<Vec<usize>> = vec![vec![]; problem.num_processors];
        let mut assigned_makespans = vec![0;problem.num_processors];

        for i in 0..problem.num_jobs {
            proc_assignments[upper_bound.assignment[i]].push(i);
            assigned_makespans[upper_bound.assignment[i]] += problem.job_sizes[i];
        }
        
        loop {
            let first_proc: usize = rng.gen_range(1..problem.num_processors);
            let second_proc = rng.gen_range(0..first_proc);
            
            self.improve_procs(problem, first_proc, second_proc, &mut proc_assignments, &mut assigned_makespans, &mut rng, lower_bound);
            let makespan = *assigned_makespans.iter().max().unwrap();
            if timeout.time_finished() || makespan == lower_bound {
                break;
            }
        }

        let mut assignments = vec![0; problem.num_jobs];
        for i in 0..problem.num_processors {
            for job in &proc_assignments[i] {
                assignments[*job] = i;
            }
        }
        let makespan = *assigned_makespans.iter().max().unwrap();

        return (lower_bound, Some(Solution { makespan: makespan, assignment: assignments }))
    }
}