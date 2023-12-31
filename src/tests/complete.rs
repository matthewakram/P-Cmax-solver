// cargo test -r --test-threads=1 --features encoding_class_instances

#[cfg(test)]
mod tests {
    use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{
                lifting, max_job_size, middle, pigeon_hole,
                sss_bound_tightening::{self},
            },
            upper_bounds::{lpt, lptp, lptpp, mss},
        },
        common::timeout::Timeout,
        encoding::{
            ilp_encoder::ILPEncoder,
            ilp_encoding::{mehdi_nizar::MehdiNizarEncoder, mehdi_nizar_original::MehdiNizarOriginalEncoder},
            sat_encoder::Encoder,
            sat_encoding::{
                basic_encoder::BasicEncoder, bdd_inter_comp::BddInterComp,
                binmerge_inter::BinmergeInterEncoder, binmerge_native::BinmergeEncoder,
                binmerge_simp::BinmergeSimpEncoder, pb_bdd_inter::PbInter,
                pb_bdd_inter_better::PbInterDyn, pb_bdd_native::PbNativeEncoder,
                pb_bdd_pysat::PbPysatEncoder, precedence_encoder::Precedence, mehdi_nizar_sat::MehdiNizarSatEncoder,
            },
        },
        input_output::{self},
        makespan_scheduling::linear_makespan::LinearMakespan,
        solvers::{
            ilp_solver::gurobi::Gurobi,
            sat_solver::{
                kissat::Kissat,
                multi_sat_solver_manager::MultiSatSolverManager,
                sat_solver_manager,
            },
            solver_manager::SolverManager, branch_and_bound::branch_and_bound::BranchAndBound,
        },
    };
    use std::{
        fs::{self, File},
        io::Write,
    };

    fn test_file(encoder: Box<dyn Encoder>, file_name: &String) -> Option<String> {
        let instance = input_output::from_file::read_from_file(file_name);
        let total_timeout_f64: f64 = 100.0;
        let precomputation_timeout = 20.0;

        // --------------CALCULATING BOUNDS--------------
        let precomp_timeout = Timeout::new(precomputation_timeout);

        let bounds: Vec<Box<dyn Bound>> = vec![
            Box::new(pigeon_hole::PigeonHole {}),
            Box::new(max_job_size::MaxJobSize {}),
            Box::new(middle::MiddleJobs {}),
            Box::new(lpt::LPT {}),
            Box::new(lptp::Lptp {}),
            //Box::new(martello_toth::MartelloToth {}),
            Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
            Box::new(lptpp::Lptpp {}),
            Box::new(lifting::Lifting::new_deterministic(1)),
            Box::new(mss::MSS::new_deterministic(4)),
        ];

        let (mut lower_bound, mut upper_bound) = (0, None);

        for i in 0..bounds.len() {
            let bound = &bounds[i];
            (lower_bound, upper_bound) =
                bound.bound(&instance, lower_bound, upper_bound, &precomp_timeout);
            if precomp_timeout.time_finished()
                || (upper_bound.is_some() && upper_bound.as_ref().unwrap().makespan == lower_bound)
            {
                break;
            }
        }
        let upper_bound = upper_bound.unwrap();

        // -------------CHECKING IF SOLUTION HAS BEEN FOUND-----------
        // We maintain that the solution is within [lower_bound, upper_bound]. Note that this is inclusive.
        let total_timeout = Timeout::new(total_timeout_f64);

        assert!(lower_bound <= upper_bound.makespan);
        if lower_bound == upper_bound.makespan {
            return Some(format!(
                "{} {} {} {} {} 0.0 0.0 0.0 0.0 0.0 0.0 0.0",
                file_name,
                total_timeout_f64 - total_timeout.remaining_time(),
                lower_bound,
                instance.num_jobs,
                instance.num_processors
            ));
        }
        println!("solving file {}", file_name);

        //--------------SOLVING---------------------------
        let mut sat_solver = sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            encoder,
        );

        let sol = sat_solver.solve(&instance, lower_bound, &upper_bound, &total_timeout, false);
        if sol.is_none() {
            return None;
        }

        if sat_solver.stats.len() < 7 {
            return Some(format!(
                "{} {} {} {} {} 0.0 0.0 0.0 0.0 0.0 0.0 0.0",
                file_name,
                total_timeout_f64 - total_timeout.remaining_time(),
                sol.as_ref().unwrap().makespan,
                instance.num_jobs,
                instance.num_processors,
            ));
        }
        return Some(format!(
            "{} {} {} {} {} {} {} {} {} {} {} {}",
            file_name,
            total_timeout_f64 - total_timeout.remaining_time(),
            sol.as_ref().unwrap().makespan,
            instance.num_jobs,
            instance.num_processors,
            sat_solver.stats.get("num_sat_calls").unwrap(),
            sat_solver.stats.get("num_unsat_calls").unwrap(),
            sat_solver.stats.get("encoding_time").unwrap(),
            sat_solver.stats.get("string_gen_time").unwrap(),
            sat_solver.stats.get("formula_write_time").unwrap(),
            sat_solver.stats.get("solve_time").unwrap(),
            sat_solver.stats.get("solution_read_time").unwrap(),
        ));
    }

    fn test_encoder(encoder: &Box<dyn Encoder>, in_dirname: &str, out_dirname: &str) {
        let paths = fs::read_dir(in_dirname).unwrap();
        let files: Vec<String> = paths
            .into_iter()
            .filter(|path| {
                path.as_ref()
                    .unwrap()
                    .path()
                    .display()
                    .to_string()
                    .ends_with(".txt")
            })
            .map(|p: Result<fs::DirEntry, std::io::Error>| p.unwrap().path().display().to_string())
            .enumerate()
            .map(|(_, x)| x)
            .collect();

        let files: Vec<(String, Box<dyn Encoder>)> = files
            .iter()
            .map(|x| (x.clone(), encoder.clone()))
            .collect::<Vec<_>>();
        let result = files
            .into_par_iter()
            //.into_iter()
            .enumerate()
            .map(|(_file_num, (path, encoder))| {
                //    println!("solving file num {}", file_num);
                test_file(encoder, &path)
            })
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<String>>();

        let result = result
            .iter()
            .map(|x| x.to_string())
            .reduce(|accum, item| accum + &"\n" + &item)
            .unwrap();

        let mut file = File::create(out_dirname).unwrap();
        file.write_all(&result.as_bytes()).unwrap();
    }

    fn test_file_solver(mut solver: Box<dyn SolverManager>, file_name: &String) -> Option<String> {
        let instance = input_output::from_file::read_from_file(file_name);
        let total_timeout_f64: f64 = 100.0;
        let precomputation_timeout = 20.0;

        // --------------CALCULATING BOUNDS--------------
        let precomp_timeout = Timeout::new(precomputation_timeout);

        let bounds: Vec<Box<dyn Bound>> = vec![
            Box::new(pigeon_hole::PigeonHole {}),
            Box::new(max_job_size::MaxJobSize {}),
            Box::new(middle::MiddleJobs {}),
            Box::new(lpt::LPT {}),
            Box::new(lptp::Lptp {}),
            //Box::new(martello_toth::MartelloToth {}),
            Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
            Box::new(lptpp::Lptpp {}),
            Box::new(lifting::Lifting::new_deterministic(1)),
            Box::new(mss::MSS::new_deterministic(4)),
        ];

        let (mut lower_bound, mut upper_bound) = (0, None);

        for i in 0..bounds.len() {
            let bound = &bounds[i];
            (lower_bound, upper_bound) =
                bound.bound(&instance, lower_bound, upper_bound, &precomp_timeout);
            if precomp_timeout.time_finished()
                || (upper_bound.is_some() && upper_bound.as_ref().unwrap().makespan == lower_bound)
            {
                break;
            }
        }
        let upper_bound = upper_bound.unwrap();
        let total_timeout = Timeout::new(total_timeout_f64);

        // -------------CHECKING IF SOLUTION HAS BEEN FOUND-----------
        // We maintain that the solution is within [lower_bound, upper_bound]. Note that this is inclusive.
        
        assert!(lower_bound <= upper_bound.makespan);
        if lower_bound == upper_bound.makespan {
            return Some(format!(
                "{} {} {} {} {} 0.0 0.0 0.0 0.0 0.0 0.0 0.0",
                file_name,
                total_timeout_f64 - total_timeout.remaining_time(),
                lower_bound,
                instance.num_jobs,
                instance.num_processors
            ));
        }
        println!("solving file {}", file_name);

        //--------------SOLVING---------------------------

        let sol = solver.solve(&instance, lower_bound, &upper_bound, &total_timeout, false);
        if sol.is_none() {
            return None;
        }

        return Some(format!(
            "{} {} {} {} {} 0.0 0.0 0.0 0.0 0.0 0.0 0.0",
            file_name,
            total_timeout_f64 - total_timeout.remaining_time(),
            sol.as_ref().unwrap().makespan,
            instance.num_jobs,
            instance.num_processors,
        ));
    }

    fn test_solver(solver: Box<dyn SolverManager>, in_dirname: &str, out_dirname: &str) {
        let paths = fs::read_dir(in_dirname).unwrap();
        let files: Vec<String> = paths
            .into_iter()
            .filter(|path| {
                path.as_ref()
                    .unwrap()
                    .path()
                    .display()
                    .to_string()
                    .ends_with(".txt")
            })
            .map(|p: Result<fs::DirEntry, std::io::Error>| p.unwrap().path().display().to_string())
            .enumerate()
            .map(|(_, x)| x)
            .collect();

        let files: Vec<(String, Box<dyn SolverManager>)> = files
            .iter()
            .map(|x| (x.clone(), solver.clone()))
            .collect::<Vec<_>>();
        let result = files
            .into_par_iter()
            //.into_iter()
            .enumerate()
            .map(|(_file_num, (path, encoder))| {
                //    println!("solving file num {}", file_num);
                test_file_solver(encoder, &path)
            })
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<String>>();

        let result = result
            .iter()
            .map(|x| x.to_string())
            .reduce(|accum, item| accum + &"\n" + &item)
            .unwrap();

        let mut file = File::create(out_dirname).unwrap();
        file.write_all(&result.as_bytes()).unwrap();
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_basic() {
        let mut encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        test_encoder(
            &mut encoder,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_pysat() {
        let mut a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_pysat.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_bdd_native() {
        let mut a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_bdd.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_inter() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::new()), 2));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_dyninter() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInterDyn::new()), 1));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_inter+.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_compinter() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(BddInterComp::new()), 1));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_intercomp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_binmerge() {
        let mut a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BinmergeEncoder::new()), 1));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_binmerge.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_binter() {
        let mut a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BinmergeInterEncoder::new()), 1));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_binmerge_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_binsimp() {
        let mut a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BinmergeSimpEncoder::new()), 1));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_binmerge_simp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_mehdi_sat() {
        let mut a: Box<dyn Encoder> = Box::new(MehdiNizarSatEncoder::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_mehdi_nizar_sat.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_multi() {
        let a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BinmergeEncoder::new()), 1));
        let b: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(BddInterComp::new()), 1));
        let solver: Box<dyn SolverManager> = Box::new(MultiSatSolverManager {
            sat_solver: Box::new(Kissat::new()),
            unsat_solver: Box::new(Kissat::new()),
            makespan_scheduler: Box::new(LinearMakespan {}),
            sat_encoder: a,
            unsat_encoder: b,
        });
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_multi.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_ilp() {
        let a: Box<dyn ILPEncoder> = Box::new(MehdiNizarEncoder::new());
        let solver = Box::new(Gurobi::new(a));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_mehdi_nizar.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_original_ilp() {
        let a: Box<dyn ILPEncoder> = Box::new(MehdiNizarOriginalEncoder::new());
        let solver = Box::new(Gurobi::new(a));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_mehdi_nizar_original.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_b_and_b() {
        let solver = Box::new(BranchAndBound::new());
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_branch_and_bound.txt",
        )
    }



    #[test]
    #[ignore]
    pub fn complete_test_franca_basic() {
        let mut encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        test_encoder(
            &mut encoder,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_pysat() {
        let mut a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_binmerge.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_bdd_native() {
        let mut a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        test_encoder(
            &mut a,
            //"./bench/class_instances/",
            //"./bench/class_instances/"
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_bdd.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_inter() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::new()), 2));
        test_encoder(
            &mut a,
            //"./bench/class_instances/",
            //"./bench/class_instances/"
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_pinter() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInterDyn::new()), 2));
        test_encoder(
            &mut a,
            //"./bench/class_instances/",
            //"./bench/class_instances/"
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_inter+.txt",
        )
    }
}
