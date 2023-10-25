

use crate::{problem_instance::problem_instance::ProblemInstance, bounds::bound::Bound};



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

fn bound_instance(instance: ProblemInstance, lower_bound: usize, upper_bound: usize) -> usize{
    
    if upper_bound <= lower_bound {
        return lower_bound;
    }
    // TODO: more expensive bounding operations here
    return lower_bound;
}

impl Bound for Lifting{
    fn bound(&self, problem: &ProblemInstance, lower_bound: usize, upper_bound: Option<crate::problem_instance::solution::Solution>) -> (usize, Option<crate::problem_instance::solution::Solution>) {
        let instances_to_bound: Vec<ProblemInstance> = get_instances(problem);
        let current_upper_bound = upper_bound.as_ref().unwrap().makespan;

        let mut best_bound = lower_bound;
        for instance in instances_to_bound {
            let bound = bound_instance(instance, best_bound, current_upper_bound);
            best_bound = best_bound.max(bound);
        }
        return (best_bound, upper_bound);
    }
}