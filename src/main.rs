use std::{env, vec};

mod bounds;
mod problem_instance;
mod input_output;
mod problem_simplification;
mod encoding;
mod solvers;

use bounds::upper_bounds::{lpt, upper_bound};
use bounds::lower_bounds::*;


use crate::bounds::lower_bounds;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];
    println!("{}", file_name);
    let instance = input_output::from_file::read_from_file(file_name);

    let initial_upper_bounds: Vec<Box<dyn upper_bound::InitialUpperBound>> = vec![Box::new(lpt::LPT{})];
    let initial_lower_bound:  Vec<Box<dyn lower_bound::LowerBound>> = vec![
        Box::new(pigeon_hole::PigeonHole{}), 
        Box::new(lower_bounds::lpt::LPT{}),
        Box::new(lower_bounds::max_job_size::MaxJobSize{})
        ];

    let initial_upper_bound = initial_upper_bounds.iter().map(|x| x.as_ref().get_upper_bound(&instance)).min_by_key(|x| x.makespan).unwrap();

    let initial_lower_bound: Vec<usize> = initial_lower_bound.iter().map(|x| x.as_ref().get_lower_bound(&instance)).collect();
    println!("initial lower bounds {:?}", initial_lower_bound);
    let initial_lower_bound = *initial_lower_bound.iter().max().unwrap();

    // We maintain that the solution is within [lower_bound, upper_bound]. Note that this is inclusive.

    let solution = instance.finalize_solution(initial_upper_bound);
    println!("{}", solution);

    assert!(initial_lower_bound <= solution.makespan);
    if initial_lower_bound == solution.makespan {
        //TODO: this
        println!("solution found");
    }



}
