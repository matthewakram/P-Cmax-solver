

use crate::{problem_instance::{problem_instance::ProblemInstance, solution::Solution}, bounds::{bound::Bound, upper_bounds::{lpt, lptp, lptpp}}, encoding::{encoder::Encoder, basic_with_precedence::Precedence, basic_encoder::BasicEncoder}, solvers::sat_solver::{sat_solver_manager, kissat}, makespan_scheduling::linear_makespan::LinearMakespan};

use super::{pigeon_hole, max_job_size, middle, sss_bound_tightening};



// TODO: this

pub struct Lifting{
    
}

fn lambda(n: usize, m: usize, k: usize)-> usize{
    let n_div_m = n/m;
    return k*n_div_m + k.min(n-n_div_m*m);
}

fn get_required_l_s(n:usize, m: usize, k:usize) -> Vec<usize> {
    return (1..n+1).into_iter().filter(|l| (l-k) % m == 0).collect();
}

fn get_instances(instance: &ProblemInstance) -> Vec<ProblemInstance>{
    let n = instance.num_jobs;
    let m = instance.num_processors;
    let mut instances_to_bound: Vec<ProblemInstance> = vec![];
    for k in 1..m+1 {
        let required_l: Vec<usize> = get_required_l_s(n, m, k);

        for l in required_l {
            let num_required_jobs: usize = lambda(l, m, k);
            let reduced_jobs: Vec<usize> = (instance.job_sizes[0..l])[l-num_required_jobs..l].to_vec();
            instances_to_bound.push(ProblemInstance::new(k, num_required_jobs, reduced_jobs))
        }
    }
    return instances_to_bound;    
}

fn bound_instance(instance: &ProblemInstance, lower_bound: usize, upper_bound: usize) -> (usize, Solution){
    let bounds: Vec<Box<dyn Bound>> = vec![
        Box::new(pigeon_hole::PigeonHole {}),
        Box::new(max_job_size::MaxJobSize {}),
        Box::new(middle::MiddleJobs {}),
        Box::new(lpt::LPT {}),
        Box::new(lptp::Lptp {}),
        Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
        Box::new(lptpp::Lptpp {}),
    ];

    let (mut new_lower_bound, mut new_upper_bound) = (0, None);
    for i in 0..bounds.len() {
        let bound = &bounds[i];
        (new_lower_bound, new_upper_bound) = bound.bound(&instance, lower_bound, new_upper_bound);
        if new_upper_bound.is_some() && (new_upper_bound.as_ref().unwrap().makespan <= lower_bound || new_upper_bound.as_ref().unwrap().makespan == new_lower_bound){
            break;
        }
    }
    return (new_lower_bound, new_upper_bound.unwrap());
}

impl Bound for Lifting{
    fn bound(&self, problem: &ProblemInstance, lower_bound: usize, upper_bound: Option<crate::problem_instance::solution::Solution>) -> (usize, Option<crate::problem_instance::solution::Solution>) {
        let instances_to_bound: Vec<ProblemInstance> = get_instances(problem);
        let current_upper_bound = upper_bound.as_ref().unwrap().makespan;

        let mut best_bound = lower_bound;
        let mut solved_exactly = 0;
        let mut unsolved_instances: Vec<ProblemInstance> = vec![];
        let mut bounds_unsolved_instances: Vec<(usize, Solution)> = vec![];
        for instance in &instances_to_bound {
            let (lower, upper) = bound_instance(instance, best_bound, current_upper_bound);
            best_bound = best_bound.max(lower);
            if lower >= upper.makespan {
                solved_exactly += 1;
            }else {
                unsolved_instances.push(instance.clone());
                bounds_unsolved_instances.push((lower, upper));
            }
        }
        println!("solved {}/{}", solved_exactly, instances_to_bound.len());
        if best_bound == upper_bound.as_ref().unwrap().makespan {
            return (best_bound, upper_bound);
        }

        let encoder: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(BasicEncoder::new())));
        let mut sat_solver = sat_solver_manager::SatSolverManager {
            sat_solver: Box::new(kissat::Kissat {}),
            makespan_scheduler: Box::new(LinearMakespan {}),
            encoder,
        };
        // we know we might still be able to improve the lower bound, and the unsolved instances are the key to that
        for i in 0..unsolved_instances.len() {
            let instance = &unsolved_instances[i];
            let (lower, upper) = bounds_unsolved_instances[i].clone();
            let upper = if upper_bound.is_none() || upper.makespan <= upper_bound.as_ref().unwrap().makespan {&upper} else {upper_bound.as_ref().unwrap()};

            let sol = sat_solver.solve(&instance, lower, upper, 5.0, false);
            if sol.is_some() {
                best_bound = best_bound.max(sol.unwrap().makespan);
            }
        }
        return (best_bound, upper_bound);
    }
}