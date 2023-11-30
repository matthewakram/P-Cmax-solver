use crate::{
    bounds::bound::Bound,
    common::timeout::Timeout,
    problem_instance::{problem_instance::ProblemInstance, solution::Solution},
};
use fraction::Fraction;

pub struct FeketeSchepers {}

fn lp(k: usize, problem: &ProblemInstance, makespan: usize) -> bool {
    let job_sizes = &problem.job_sizes;
    // these are normalized to the range [0,k]
    let normalized_job_sizes: Vec<Fraction> = job_sizes
        .iter()
        .map(|x: &usize| {
            if ((k + 1) * x) % makespan != 0 {
                let rounded = Fraction::from(((k + 1) * x) % makespan/ makespan);
                rounded / Fraction::from(k)
            } else {
                Fraction::from(*x / makespan)
            }
        })
        .collect();

    let normalized_sum: Fraction = normalized_job_sizes.iter().sum();
    

    let mut upper_pointer = 0;
    let mut lower_pointer = normalized_job_sizes.len() - 1;

    let mut last_sum = normalized_sum;
    let mut max_sum = normalized_sum;
    let mut epsilon = 0;

    while epsilon <= makespan / 2 {
        if lower_pointer == upper_pointer {
            break;
        }
        epsilon = (makespan - job_sizes[upper_pointer]).min(job_sizes[lower_pointer]) + 1;

        while job_sizes[upper_pointer] > makespan - epsilon {
            last_sum -= normalized_job_sizes[upper_pointer];
            last_sum += 1;
            upper_pointer += 1;

            if lower_pointer == upper_pointer {
                break;
            }
        }
        if lower_pointer == upper_pointer {
            break;
        }
        while job_sizes[lower_pointer] < epsilon {
            last_sum -= normalized_job_sizes[lower_pointer];
            lower_pointer -= 1;

            if lower_pointer == upper_pointer {
                break;
            }
        }

        if last_sum > max_sum {
            max_sum = last_sum;
        }
        if lower_pointer == upper_pointer {
            break;
        }
    }

    let limit = Fraction::from(problem.num_processors);

    return max_sum <= limit;
}

fn is_feasable(problem: &ProblemInstance, makespan: usize) -> bool {
    let k = 100;
    for k in 2..k + 1 {
        if !lp(k, problem, makespan) {
            return false;
        }
    }
    return true;
}

impl Bound for FeketeSchepers {
    fn bound(
        &self,
        problem: &crate::problem_instance::problem_instance::ProblemInstance,
        lower_bound: usize,
        upper_bound: Option<Solution>,
        timeout: &Timeout,
    ) -> (usize, Option<Solution>) {
        let mut lower_bound = lower_bound;
        loop {
            if (upper_bound.is_some() && lower_bound == upper_bound.as_ref().unwrap().makespan)
                || timeout.time_finished()
            {
                return (lower_bound, upper_bound);
            }

            let it_is = is_feasable(problem, lower_bound);
            if it_is {
                return (lower_bound, upper_bound);
            }

            lower_bound += 1;
        }
    }
}
