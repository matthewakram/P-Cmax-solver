use std::{env, vec};

mod bdd;
mod bounds;
mod common;
mod encoding;
mod input_output;
mod makespan_scheduling;
mod precedence_relations;
mod problem_instance;
mod problem_simplification;
mod solvers;
mod randomized_checkers;
mod _perf_tests;
//mod feasability_checking;
extern crate bitvec;

use bounds::lower_bounds::*;
use bounds::upper_bounds::lpt;

use crate::bounds::bound::Bound;
use crate::bounds::upper_bounds::{lptp, lptpp};
use crate::common::common::IndexOf;
use crate::common::timeout::Timeout;
use crate::encoding::basic_encoder::BasicEncoder;
//use crate::encoding::basic_with_fill_up::BasicWithFillUp;
use crate::encoding::basic_with_precedence::Precedence;
use crate::encoding::encoder::Encoder;
use crate::encoding::pb_bdd_inter::PbInter;
use crate::encoding::pb_bdd_inter_better::PbInterDyn;
//use crate::encoding::fill_up_lite::FillUpLite;
use crate::encoding::pb_bdd_native::PbNativeEncoder;
use crate::encoding::pb_bdd_pysat::PbPysatEncoder;
use crate::makespan_scheduling::linear_makespan::LinearMakespan;
use crate::solvers::sat_solver::{kissat, sat_solver_manager};

fn main() {
    // --------------READING THE INPUT--------------
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];
    let instance = input_output::from_file::read_from_file(file_name);

    let timeout_given = args.index_of(&"-t".to_string());
    let total_timeout  = if timeout_given.is_none() {
        60.0
    } else {
        args[timeout_given.unwrap() + 1].parse::<f64>().unwrap()
    };
    let precomputation_timeout = total_timeout / 5.0;

    // --------------CALCULATING BOUNDS--------------
    let precomp_timeout = Timeout::new(precomputation_timeout);
    let total_timeout = Timeout::new(total_timeout);
    let bounds: Vec<Box<dyn Bound>> = vec![
        Box::new(pigeon_hole::PigeonHole {}),
        Box::new(max_job_size::MaxJobSize {}),
        Box::new(middle::MiddleJobs {}),
        Box::new(lpt::LPT {}),
        Box::new(lptp::Lptp {}),
        //Box::new(martello_toth::MartelloToth {}),
        Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
        Box::new(lptpp::Lptpp {}),
        Box::new(lifting::Lifting {}),
    ];

    let (mut lower_bound, mut upper_bound) = (0, None);
    //TODO; make this dynamic
    for i in 0..bounds.len() {
        let bound = &bounds[i];
        (lower_bound, upper_bound) = bound.bound(&instance, lower_bound, upper_bound, &precomp_timeout);
        println!("lower: {} upper {}", lower_bound, if upper_bound.is_some() {upper_bound.as_ref().unwrap().makespan} else {0});
        if precomp_timeout.time_finished(){
            break;
        }
    }
    let upper_bound = upper_bound.unwrap();

    // -------------CHECKING IF SOLUTION HAS BEEN FOUND-----------
    // We maintain that the solution is within [lower_bound, upper_bound]. Note that this is inclusive.

    assert!(lower_bound <= upper_bound.makespan);
    if lower_bound == upper_bound.makespan {
        //TODO: this
        println!("solution found {}", upper_bound.makespan);
        //println!("jobs sizes {:?}", instance.job_sizes);
        return;
    }

    //--------------SOLVING---------------------------
    let encoder: Box<dyn Encoder>;
    if args.contains(&"-fur".to_string()) {
        //encoder = Box::new(BasicWithFillUp::new());
        panic!("Fur is no longer supported")
    } else if args.contains(&"-furlite".to_string()) {
        panic!("Furlite is also no longer supported")
        //encoder = Box::new(FillUpLite::new());
    } else if args.contains(&"-pysat".to_string()) && args.contains(&"-prec".to_string()) {
        encoder = Box::new(Precedence::new(Box::new(PbPysatEncoder::new()), 2));
    } else if args.contains(&"-pysat".to_string()) {
        encoder = Box::new(PbPysatEncoder::new());
    } else if args.contains(&"-bdd".to_string()) && args.contains(&"-prec".to_string()) {
        encoder = Box::new(Precedence::new(Box::new(PbNativeEncoder::new()), 2));
    } else if args.contains(&"-bdd".to_string()) {
        encoder = Box::new(PbNativeEncoder::new());
    } else if args.contains(&"-inter".to_string()) && args.contains(&"-prec".to_string()) {
        encoder = Box::new(Precedence::new(Box::new(PbInter::new()), 2));
    } else if args.contains(&"-inter".to_string()) {
        encoder = Box::new(PbInter::new());
    } else if args.contains(&"-inter+".to_string()) && args.contains(&"-prec".to_string()) {
        encoder = Box::new(Precedence::new(Box::new(PbInterDyn::new()), 2));
    }else if args.contains(&"-basic".to_string()) && args.contains(&"-prec".to_string()) {
        encoder = Box::new(Precedence::new(Box::new(BasicEncoder::new()), 2));
    } else if args.contains(&"-basic".to_string()) {
        encoder = Box::new(BasicEncoder::new());
    } else {
        panic!("need to specify one of the given options")
    }

    let mut sat_solver = sat_solver_manager::SatSolverManager {
        sat_solver: Box::new(kissat::Kissat {}),
        makespan_scheduler: Box::new(LinearMakespan {}),
        encoder,
    };
    
    let sol = sat_solver.solve(&instance, lower_bound, &upper_bound, &total_timeout, true).unwrap();
    let final_solution = instance.finalize_solution(sol);
    println!("solution found {}", final_solution.makespan);
}
