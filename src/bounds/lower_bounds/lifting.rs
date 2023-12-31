use rand::Rng;

use crate::{
    bounds::{
        bound::Bound,
        upper_bounds::{lpt, lptp, lptpp, mss},
    },
    common::timeout::Timeout,
    encoding::{sat_encoder::Encoder, sat_encoding::{precedence_encoder::Precedence, pb_bdd_inter_better::PbInterDyn}
    },
    makespan_scheduling::linear_makespan::LinearMakespan,
    problem_instance::{problem_instance::ProblemInstance, solution::Solution},
    solvers::{sat_solver::{kissat::Kissat, sat_solver_manager}, solver_manager::SolverManager},
};

use super::{fs, max_job_size, middle, pigeon_hole, sss_bound_tightening};

pub struct Lifting {
    seed: u64,
}

impl Lifting {
    #[cfg(test)]
    pub fn new_deterministic(seed: u64) -> Lifting {
        return Lifting { seed };
    }

    pub fn new() -> Lifting {
        return Lifting {
            seed: rand::thread_rng().gen(),
        };
    }

    fn bound_instance(
        &self,
        instance: &ProblemInstance,
        lower_bound: usize,
        remaining_time: &Timeout,
    ) -> (usize, Solution) {
        let bounds: Vec<Box<dyn Bound>> = vec![
            Box::new(pigeon_hole::PigeonHole {}),
            Box::new(max_job_size::MaxJobSize {}),
            Box::new(middle::MiddleJobs {}),
            Box::new(lpt::LPT {}),
            Box::new(lptp::Lptp {}),
            Box::new(fs::FeketeSchepers {}),
            Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
            Box::new(lptpp::Lptpp {}),
            Box::new(mss::MSS::new_deterministic(self.seed)),
        ];

        let (mut new_lower_bound, mut new_upper_bound) = (0, None);
        for i in 0..bounds.len() {
            if remaining_time.time_finished() {
                break;
            }
            let bound = &bounds[i];
            (new_lower_bound, new_upper_bound) =
                bound.bound(&instance, new_lower_bound, new_upper_bound, remaining_time);

            if new_upper_bound.is_some()
                && (new_upper_bound.as_ref().unwrap().makespan <= lower_bound
                    || new_upper_bound.as_ref().unwrap().makespan == new_lower_bound)
            {
                break;
            }
        }

        if new_lower_bound > new_upper_bound.as_ref().unwrap().makespan {
            println!(
                "error with lower {} and upper  {}",
                new_lower_bound,
                new_upper_bound.as_ref().unwrap().makespan
            );
            println!("{:?}", instance);
        }

        return (new_lower_bound, new_upper_bound.unwrap());
    }
}

fn lambda(n: usize, m: usize, k: usize) -> usize {
    let n_div_m = n / m;
    return k * n_div_m + k.min(n - n_div_m * m);
}

fn get_required_l_s(n: usize, m: usize, k: usize) -> Vec<usize> {
    return (1..n + 1)
        .into_iter()
        .filter(|l| (l - k) % m == 0)
        .collect();
}

fn get_instances(instance: &ProblemInstance) -> Vec<ProblemInstance> {
    let n = instance.num_jobs;
    let m = instance.num_processors;
    let mut instances_to_bound: Vec<ProblemInstance> = vec![];
    for k in 1..m + 1 {
        let required_l: Vec<usize> = get_required_l_s(n, m, k);

        for l in required_l {
            let num_required_jobs: usize = lambda(l, m, k);
            let reduced_jobs: Vec<usize> =
                (instance.job_sizes[0..l])[l - num_required_jobs..l].to_vec();
            instances_to_bound.push(ProblemInstance::new(k, num_required_jobs, reduced_jobs))
        }
    }
    return instances_to_bound;
}

impl Bound for Lifting {
    fn bound(
        &self,
        problem: &ProblemInstance,
        lower_bound: usize,
        upper_bound: Option<crate::problem_instance::solution::Solution>,
        timeout: &Timeout,
    ) -> (usize, Option<crate::problem_instance::solution::Solution>) {
        let instances_to_bound: Vec<ProblemInstance> = get_instances(problem);

        let mut best_bound = lower_bound;
        //let mut solved_exactly = 0;
        let mut unsolved_instances: Vec<ProblemInstance> = vec![];
        let mut bounds_unsolved_instances: Vec<(usize, Solution)> = vec![];
        let bounding_timeout = timeout.clone_with_new_timeout(timeout.remaining_time() / 4.0);
        let mut num_instances_remaining = instances_to_bound.len() as f64;
        for instance in &instances_to_bound {
            if bounding_timeout.time_finished() {
                break;
            }

            let instance_bound_time = bounding_timeout.clone_with_new_timeout(
                bounding_timeout.remaining_time() / num_instances_remaining,
            );
            num_instances_remaining -= 1.0;
            let (lower, upper) = self.bound_instance(instance, best_bound, &instance_bound_time);

            best_bound = best_bound.max(lower);

            assert!(lower <= upper.makespan);

            if lower == upper.makespan || upper.makespan <= lower_bound {
                //solved_exactly += 1;
            } else {
                unsolved_instances.push(instance.clone());
                bounds_unsolved_instances.push((lower, upper));
            }
        }
        if best_bound == upper_bound.as_ref().unwrap().makespan || timeout.time_finished() {
            return (best_bound, upper_bound);
        }

        let encoder: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInterDyn::new()), 2));
        let mut sat_solver = sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            encoder,
        );

        // we know we might still be able to improve the lower bound, and the unsolved instances are the key to that
        let mut num_instances_remaining = unsolved_instances.len() as f64;
        for i in 0..unsolved_instances.len() {
            if timeout.time_finished() {
                break;
            }

            let instance = &unsolved_instances[i];
            let (_lower, upper) = bounds_unsolved_instances[i].clone();
            let upper = if upper_bound.is_none()
                || upper.makespan <= upper_bound.as_ref().unwrap().makespan
            {
                &upper
            } else {
                upper_bound.as_ref().unwrap()
            };
            let sol = sat_solver.solve(
                &instance,
                _lower,
                upper,
                &Timeout::new(
                    (10.0 as f64).min(timeout.remaining_time() / num_instances_remaining),
                ),
                false,
            );
            num_instances_remaining -= 1.0;
            if sol.is_some() {
                best_bound = best_bound.max(sol.unwrap().makespan);
            } else {
                println!(
                    "could not solve instance number {}/{}",
                    i,
                    unsolved_instances.len()
                );
            }
        }
        return (best_bound, upper_bound);
    }
}
