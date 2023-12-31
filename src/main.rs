use std::{env, vec};

use p_cmax_solver::encoding::cplex_model_encoding::pinar_seyda::PinarSeyda;
use p_cmax_solver::encoding::ilp_encoding::mehdi_nizar::MehdiNizarEncoder;
use p_cmax_solver::encoding::sat_encoding::basic_encoder::BasicEncoder;
use p_cmax_solver::encoding::sat_encoding::bdd_inter_comp::BddInterComp;
use p_cmax_solver::encoding::sat_encoding::binmerge_inter::BinmergeInterEncoder;
use p_cmax_solver::encoding::sat_encoding::pb_bdd_inter::PbInter;
use p_cmax_solver::encoding::sat_encoding::pb_bdd_inter_better::PbInterDyn;
use p_cmax_solver::encoding::sat_encoding::pb_bdd_native::PbNativeEncoder;
use p_cmax_solver::encoding::sat_encoding::pb_bdd_pysat::PbPysatEncoder;
use p_cmax_solver::encoding::sat_encoding::precedence_encoder::Precedence;
use p_cmax_solver::solvers::cp_solver::cplex_manager::CPELXSolver;
use p_cmax_solver::solvers::ilp_solver::gurobi::Gurobi;
use p_cmax_solver::solvers::solver_manager::SolverManager;
use p_cmax_solver::{bounds, input_output};

use bounds::lower_bounds::*;
use bounds::upper_bounds::lpt;

use p_cmax_solver::bounds::bound::Bound;
use p_cmax_solver::bounds::upper_bounds::{lptp, lptpp, mss};
use p_cmax_solver::common::common::IndexOf;
use p_cmax_solver::common::timeout::Timeout;
use p_cmax_solver::encoding::sat_encoder::Encoder;
use p_cmax_solver::makespan_scheduling::linear_makespan::LinearMakespan;
use p_cmax_solver::solvers::sat_solver::kissat::Kissat;
use p_cmax_solver::solvers::sat_solver::{multi_sat_solver_manager, sat_solver_manager};

fn main() {
    // --------------READING THE INPUT--------------
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];
    let instance = input_output::from_file::read_from_file(file_name);

    let timeout_given = args.index_of(&"-t".to_string());
    let total_timeout = if timeout_given.is_none() {
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
        Box::new(fs::FeketeSchepers {}),
        Box::new(lpt::LPT {}),
        Box::new(lptp::Lptp {}),
        Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
        Box::new(lptpp::Lptpp {}),
        Box::new(lifting::Lifting::new()),
        Box::new(mss::MSS::new()),
    ];

    let (mut lower_bound, mut upper_bound) = (1, None);
    //TODO; make this dynamic
    for i in 0..bounds.len() {
        let bound = &bounds[i];
        (lower_bound, upper_bound) =
            bound.bound(&instance, lower_bound, upper_bound, &precomp_timeout);
        println!(
            "lower: {} upper {}",
            lower_bound,
            if upper_bound.is_some() {
                upper_bound.as_ref().unwrap().makespan
            } else {
                0
            }
        );
        if precomp_timeout.time_finished()
            || (upper_bound.is_some() && upper_bound.as_ref().unwrap().makespan == lower_bound)
        {
            break;
        }
    }
    let mut upper_bound = upper_bound.unwrap();
    upper_bound.makespan += 1;

    // -------------CHECKING IF SOLUTION HAS BEEN FOUND-----------
    // We maintain that the solution is within [lower_bound, upper_bound]. Note that this is inclusive.

    assert!(lower_bound <= upper_bound.makespan);
    if lower_bound == upper_bound.makespan {
        //TODO: this
        println!("solution found {}", upper_bound.makespan);
        return;
    }

    if args.contains(&"-cplex".to_string()) {
        let encoder = Box::new(PinarSeyda::new());
        let mut solver = CPELXSolver::new(encoder);

        let sol = solver
            .solve(&instance, lower_bound, &upper_bound, &total_timeout, true)
            .unwrap();
        let final_solution = instance.finalize_solution(sol);
        println!("solution found {}", final_solution.makespan);
        return;
    }
    if args.contains(&"-ilp".to_string()) {
        let encoder = Box::new(MehdiNizarEncoder::new());
        let mut solver = Gurobi::new(encoder);

        let sol = solver
            .solve(&instance, lower_bound, &upper_bound, &total_timeout, true)
            .unwrap();
        let final_solution = instance.finalize_solution(sol);
        println!("solution found {}", final_solution.makespan);
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
    } else if args.contains(&"-basic".to_string()) && args.contains(&"-prec".to_string()) {
        encoder = Box::new(Precedence::new(Box::new(BasicEncoder::new()), 2));
    } else if args.contains(&"-basic".to_string()) {
        encoder = Box::new(BasicEncoder::new());
    } else if args.contains(&"-intercomp".to_string()) && args.contains(&"-prec".to_string()) {
        encoder = Box::new(Precedence::new(Box::new(BddInterComp::new()), 1));
    } else if args.contains(&"-binmerge".to_string()) {
        encoder = Box::new(BinmergeInterEncoder::new());
    } else {
        panic!("need to specify one of the given options")
    }

    if args.contains(&"-test_new".to_string()) {
        let mut multisat_solver = multi_sat_solver_manager::MultiSatSolverManager {
            sat_solver: Box::new(Kissat::new()),
            unsat_solver: Box::new(Kissat::new()),
            makespan_scheduler: Box::new(LinearMakespan {}),
            sat_encoder: Box::new(PbPysatEncoder::new()),
            unsat_encoder: Box::new(PbNativeEncoder::new()),
        };

        let sol = multisat_solver
            .solve(&instance, lower_bound, &upper_bound, &total_timeout, true)
            .unwrap();
        let final_solution = instance.finalize_solution(sol);
        println!("solution found {}", final_solution.makespan);
        return;
    }

    let mut sat_solver = sat_solver_manager::SatSolverManager::new(
        Box::new(Kissat::new()),
        Box::new(LinearMakespan {}),
        encoder,
    );

    let sol = sat_solver
        .solve(&instance, lower_bound, &upper_bound, &total_timeout, true)
        .unwrap();
    let final_solution = instance.finalize_solution(sol);
    println!("solution found {}", final_solution.makespan);
}
