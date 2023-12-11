
use rand::{Rng, rngs::StdRng, seq::SliceRandom, SeedableRng};

use crate::{problem_instance::{problem_instance::ProblemInstance, solution::Solution}, bounds::bound::Bound, common::{timeout::Timeout, self}};


pub struct MSS {
    seed: u64
}

impl MSS {
    pub fn new_deterministic(seed: u64) -> MSS {
        return MSS{seed};
    }

    pub fn new() -> MSS {
        return MSS { seed: rand::thread_rng().gen() }
    }
}

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
    fn improve_procs(&self, instance: &ProblemInstance, first_proc: usize, second_proc: usize, assignments: &mut Vec<Vec<usize>>, assigned_makespans: &mut Vec<usize>, rng: &mut StdRng, lower_bound: usize) {

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
        if problem.num_processors == 1 {
            return (lower_bound, upper_bound);
        }
        assert!(upper_bound.is_some());
        let upper_bound = upper_bound.unwrap();
        let mut rng: StdRng = StdRng::seed_from_u64(self.seed);

        let mut proc_assignments: Vec<Vec<usize>> = vec![vec![]; problem.num_processors];
        let mut assigned_makespans = vec![0;problem.num_processors];

        for i in 0..problem.num_jobs {
            proc_assignments[upper_bound.assignment[i]].push(i);
            assigned_makespans[upper_bound.assignment[i]] += problem.job_sizes[i];
        }

        let makespan_checker = common::common::calc_makespan(problem, &upper_bound.assignment);
        assert_eq!(makespan_checker, upper_bound.makespan);

        let mut best_makespan = upper_bound.makespan;
        let mut best_assignment = proc_assignments.clone();

        let mut num_iter_without_improvement = 0;
        let max_num_iter_without_improvement = (problem.num_processors * problem.num_processors + 10).min(100);
        let mut pertubation_amount = 1;
        let max_num_pertubations = 2 * problem.num_jobs / problem.num_processors;
        
        loop {
            let first_proc: usize = rng.gen_range(0..problem.num_processors);
            let second_proc = rng.gen_range(0..problem.num_processors - 1);
            let second_proc = if second_proc < first_proc {second_proc} else {second_proc +1};
            
            let first_proc_makespan = assigned_makespans[first_proc];
            let second_proc_makespan = assigned_makespans[second_proc];
            self.improve_procs(problem, first_proc, second_proc, &mut proc_assignments, &mut assigned_makespans, &mut rng, lower_bound);
            let makespan: usize = *assigned_makespans.iter().max().unwrap();

            // assert correctness=======
            //let mut job_counter = 0;
            //for i in 0..problem.num_processors {
            //    let mut acc_assigned_makespan = 0;
            //    for j in &proc_assignments[i] {
            //        acc_assigned_makespan += problem.job_sizes[*j];
            //        job_counter += 1;
            //    }
            //    if acc_assigned_makespan != assigned_makespans[i] {
            //        println!("ooooooooooohhhhhhhhhh nooooooooo {} {}", acc_assigned_makespan, assigned_makespans[i]);
            //    }
            //}
            //if job_counter != problem.num_jobs {
            //    println!("wrong number of jobs mate ");
            //}
            // ============

            if makespan < best_makespan {
                best_makespan = makespan;
                best_assignment = proc_assignments.clone();
            }

            if assigned_makespans[first_proc] < first_proc_makespan || assigned_makespans[second_proc] < second_proc_makespan {
                num_iter_without_improvement = 0;
            } else {
                num_iter_without_improvement += 1;
                if num_iter_without_improvement ==  max_num_iter_without_improvement{
                    // here we pertubate the solution to get out of the neighbourhood
                    //println!("pertubating");
                    let first_proc: usize = rng.gen_range(0..problem.num_processors);
                    for _ in 0..pertubation_amount {
                        if proc_assignments[first_proc].is_empty() {
                            pertubation_amount = 0;
                            break;
                        }
                        let second_proc = rng.gen_range(0..problem.num_processors - 1);
                        let second_proc = if second_proc < first_proc {second_proc} else {second_proc +1};

                        let job_to_move = proc_assignments[first_proc].pop().unwrap();
                        assigned_makespans[first_proc] -= problem.job_sizes[job_to_move];
                        assigned_makespans[second_proc] += problem.job_sizes[job_to_move];
                        proc_assignments[second_proc].push(job_to_move);
                    }
                    pertubation_amount = if pertubation_amount < max_num_pertubations {1} else {pertubation_amount +1};

                }
            }

            if timeout.time_finished() || makespan == lower_bound {
                break;
            }
        }

        let proc_assignments = best_assignment;

        let mut assignments = vec![0; problem.num_jobs];
        for i in 0..problem.num_processors {
            for job in &proc_assignments[i] {
                assignments[*job] = i;
            }
        }
        let makespan = best_makespan;
        //let asserted_makespan = common::common::calc_makespan(problem, &assignments);
        //if asserted_makespan != makespan{
        //    println!("makespan errrrrooooooooooooooooooor {} {}", asserted_makespan, makespan);
        //}

        return (lower_bound, Some(Solution { makespan: makespan, assignment: assignments }))
    }
}