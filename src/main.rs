use std::{env, vec};

mod bounds;
mod common;
mod encoding;
mod input_output;
mod makespan_scheduling;
mod problem_instance;
mod problem_simplification;
mod solvers;
mod precedence_relations;
mod bdd;
extern crate bitvec;

use bounds::lower_bounds::*;
use bounds::upper_bounds::{lpt, upper_bound};

use crate::bounds::lower_bounds;
use crate::bounds::upper_bounds::lptpp;
use crate::encoding::basic_encoder::BasicEncoder;
use crate::encoding::basic_with_fill_up::BasicWithFillUp;
use crate::encoding::basic_with_precedence::BasicWithPrecedence;
use crate::encoding::encoder::Encoder;
use crate::encoding::fill_up_lite::FillUpLite;
use crate::encoding::furlite_with_precedence::FurliteWithPrecedence;
use crate::encoding::pb_bdd_native::PbNativeEncoder;
use crate::encoding::pb_bdd_pysat::PbPysatEncoder;
use crate::makespan_scheduling::linear_makespan::LinearMakespan;
use crate::problem_instance::solution::Solution;
use crate::solvers::sat_solver::{kissat, sat_solver_manager};

fn main() {
    // -------------READING THE INPUT--------------
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];
    let instance = input_output::from_file::read_from_file(file_name);

    //-------------CALCULATING BOUNDS--------------
    

    let initial_lower_bound: Vec<Box<dyn lower_bound::LowerBound>> = vec![
        Box::new(pigeon_hole::PigeonHole {}),
        Box::new(lower_bounds::lpt::LPT {}),
        Box::new(lower_bounds::max_job_size::MaxJobSize {}),
        Box::new(lower_bounds::middle::MiddleJobs{})
    ];


    let initial_lower_bound: Vec<usize> = initial_lower_bound
        .iter()
        .map(|x| x.as_ref().get_lower_bound(&instance))
        .collect();
    println!("initial lower bounds {:?}", initial_lower_bound);

    let initial_lower_bound: usize = *initial_lower_bound.iter().max().unwrap();

    let initial_upper_bounds: Vec<Box<dyn upper_bound::InitialUpperBound>> =
    vec![Box::new(lpt::LPT {}),
    Box::new(lptpp::Lptpp {lower_bound: initial_lower_bound}),
    ];

    let initial_upper_bounds: Vec<Solution> = initial_upper_bounds
        .iter()
        .map(|x| x.as_ref().get_upper_bound(&instance))
        .collect();
    println!(
        "Initial upper bounds {:?}",
        initial_upper_bounds
            .iter()
            .map(|x| x.makespan)
            .collect::<Vec<usize>>()
    );
    let initial_upper_bound = initial_upper_bounds
        .iter()
        .min_by_key(|x| x.makespan)
        .unwrap();


    // -------------CHECKING IF SOLUTION HAS BEEN FOUND-----------
    // We maintain that the solution is within [lower_bound, upper_bound]. Note that this is inclusive.

    assert!(initial_lower_bound <= initial_upper_bound.makespan);
    if initial_lower_bound == initial_upper_bound.makespan {
        //TODO: this
        println!("solution found");
        println!("{}", initial_upper_bound);
        return;
    }

    //--------------SOLVING---------------------------
    let encoder: Box<dyn Encoder>;
    if args.contains(&"-fur".to_string()) {
        encoder = Box::new(BasicWithFillUp::new());
    } else if args.contains(&"-furlite".to_string()) {
        encoder = Box::new(FillUpLite::new());
    } else if args.contains(&"-prec".to_string()) {
        encoder = Box::new(FurliteWithPrecedence::new());
    } else if args.contains(&"-basic_prec".to_string()) {
        encoder = Box::new(BasicWithPrecedence::new());
    } else if args.contains(&"-pysat".to_string()) {
        encoder = Box::new(PbPysatEncoder::new());
    } else if args.contains(&"-bdd".to_string()) {
        encoder = Box::new(PbNativeEncoder::new());
    } else if args.contains(&"-basic".to_string())  {
        encoder = Box::new(BasicEncoder::new());
    } else {
        panic!("need to specify one of the given options")
    }

    let mut sat_solver = sat_solver_manager::SatSolverManager {
        sat_solver: Box::new(kissat::Kissat {}),
        makespan_scheduler: Box::new(LinearMakespan {}),
        encoder
    };

    let sol = sat_solver.solve(&instance, initial_lower_bound, &initial_upper_bound);
    let final_solution = instance.finalize_solution(sol);
    println!("solution found {}", final_solution.makespan);
    println!("{}", final_solution);
}
