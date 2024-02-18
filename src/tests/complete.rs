// cargo test -r --test-threads=1 --features encoding_class_instances

#[cfg(test)]
mod tests {
    use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{lifting, max_job_size, middle, pigeon_hole, sss_bound_tightening},
            upper_bounds::{lpt, lptp, lptpp, mss},
        },
        common::timeout::Timeout,
        encoding::{
            ilp_encoder::ILPEncoder,
            ilp_encoding::{
                mehdi_nizar_original::MehdiNizarOriginalEncoder,
                mehdi_nizar_prec::MehdiNizarOrderEncoder,
            },
            sat_encoder::Encoder,
            sat_encoding::{
                basic_encoder::BasicEncoder, bdd_inter_comp::BddInterComp,
                binmerge_native::BinmergeEncoder, binmerge_simp::BinmergeSimpEncoder,
                pb_bdd_native::PbNativeEncoder, pb_bdd_pysat::PbPysatEncoder,
                precedence_encoder::Precedence,
            },
        },
        input_output,
        makespan_scheduling::linear_makespan::LinearMakespan,
        solvers::{
            branch_and_bound::{
                branch_and_bound::BranchAndBound, compressed_bnb::CompressedBnB, hj::HJ,
            },
            ilp_solver::gurobi::Gurobi,
            sat_solver::{kissat::Kissat, multi_sat_solver_manager::MultiSatSolverManager, sat_solver_manager},
            solver_manager::SolverManager,
        },
    };
    use std::{
        fs::{self, File},
        io::Write, sync::{Arc, Mutex},
    };

    fn test_file_solver(
        mut solver: Box<dyn SolverManager>,
        file_name: &String,
        progress: Arc<Mutex<usize>>,
        num_total_instances: usize,
    ) -> Option<String> {
        {
            let mut p = progress.lock().unwrap();
            *p += 1;
            println!("solving {}/{}", *p, num_total_instances);
        }
        let instance = input_output::from_file::read_from_file(file_name);
        let total_timeout_f64: f64 = 900.0;
        let precomputation_timeout = 10.0;

        // --------------CALCULATING BOUNDS--------------

        let bounds: Vec<Box<dyn Bound>> = vec![
            Box::new(pigeon_hole::PigeonHole {}),
            Box::new(max_job_size::MaxJobSize {}),
            Box::new(middle::MiddleJobs {}),
            Box::new(lpt::LPT {}),
            Box::new(lptp::Lptp {}),
            Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
            Box::new(lptpp::Lptpp {}),
            Box::new(lifting::Lifting::new_deterministic(1)),
            Box::new(mss::MSS::new_deterministic(4)),
        ];

        let (mut lower_bound, mut upper_bound) = (0, None);

        for i in 0..bounds.len() {
            let precomp_timeout = Timeout::new(precomputation_timeout);
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
                "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                file_name,
                0,
                lower_bound,
                instance.num_jobs,
                instance.num_processors,
                (instance.num_jobs as f64) / (instance.num_processors as f64),
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0
            ));
        }
        //--------------SOLVING---------------------------

        let sol = solver.solve(&instance, lower_bound, &upper_bound, &total_timeout, false);
        if sol.is_none() {
          //  println!("could not solve file {}", file_name);
            return None;
        }
        //println!("solved file {}", file_name);

        let stats = solver.get_stats();
        return Some(format!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            file_name,
            total_timeout_f64 - total_timeout.remaining_time(),
            sol.as_ref().unwrap().makespan,
            instance.num_jobs,
            instance.num_processors,
            (instance.num_jobs as f64) / (instance.num_processors as f64),
            stats.get("num_sat_calls").unwrap_or(&0.0),
            stats.get("num_unsat_calls").unwrap_or(&0.0),
            stats.get("encoding_time").unwrap_or(&0.0),
            stats.get("string_gen_time").unwrap_or(&0.0),
            stats.get("formula_write_time").unwrap_or(&0.0),
            stats.get("solve_time").unwrap_or(&0.0),
            stats.get("solution_read_time").unwrap_or(&0.0),
            stats.get("mem_used").unwrap_or(&0.0),
            stats.get("ret_construction_time").unwrap_or(&0.0),
            stats.get("solution_read_time").unwrap_or(&0.0),
            stats.get("num_nodes_explored").unwrap_or(&0.0),
        ));
    }

    fn test_solver(solver: Box<dyn SolverManager>, in_dirname: &str, out_dirname: &str) {
        if fs::metadata(out_dirname).is_ok() {
            return;
        }
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

        let progress: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let num_instances = files.len();
        let result = files
            .into_par_iter()
            //.into_iter()
            .enumerate()
            .map(|(_file_num, (path, encoder))| {
                //    println!("solving file num {}", file_num);
                test_file_solver(encoder, &path, progress.clone(), num_instances)
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
        let encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            encoder,
        ));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_basic() {
        let encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            encoder,
        ));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_instances_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_basic() {
        let encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            encoder,
        ));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_pysat() {
        let a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_pysat.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_pysat() {
        let a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_pysat.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_pysat() {
        let a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_pysat.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_bdd_native() {
        let a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_bdd.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_bdd_native() {
        let a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_instances_bdd.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_bdd_native() {
        let a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_bdd.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_bdd_prec() {
        let a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbNativeEncoder::new()), 1));
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_bdd_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_bdd_prec() {
        let a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbNativeEncoder::new()), 1));
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_instances_bdd_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_bdd_prec() {
        let a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbNativeEncoder::new()), 1));
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_bdd_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_bdd_inter_only() {
        let a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BddInterComp::new_inter_only()), 1));
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_inter_only.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_bdd_inter_only() {
        let a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BddInterComp::new_inter_only()), 1));
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_instances_inter_only.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_bdd_inter_only() {
        let a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BddInterComp::new_inter_only()), 1));
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_inter_only.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_bdd_intercomp() {
        let a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BddInterComp::new_inter_only()), 1));
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_intercomp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_bdd_intercomp() {
        let a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BddInterComp::new_inter_only()), 1));
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_instances_intercomp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_bdd_intercomp() {
        let a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BddInterComp::new_inter_only()), 1));
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_intercomp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_binmerge() {
        let a: Box<dyn Encoder> = Box::new(BinmergeEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_binmerge.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_binmerge() {
        let a: Box<dyn Encoder> = Box::new(BinmergeEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_instances_binmerge.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_binmerge() {
        let a: Box<dyn Encoder> = Box::new(BinmergeEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_instances_binmerge.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_binsimp() {
        let a: Box<dyn Encoder> = Box::new(BinmergeSimpEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_binmerge_simp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_binsimp() {
        let a: Box<dyn Encoder> = Box::new(BinmergeSimpEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_instances_binmerge_simp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_binsimp() {
        let a: Box<dyn Encoder> = Box::new(BinmergeSimpEncoder::new());
        let solver = Box::new(sat_solver_manager::SatSolverManager::new(
            Box::new(Kissat::new()),
            Box::new(LinearMakespan {}),
            a,
        ));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_instances_binmerge_simp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_fur_ilp() {
        let a: Box<dyn ILPEncoder> = Box::new(MehdiNizarOrderEncoder::new());
        let solver = Box::new(Gurobi::new(a));
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_mehdi_nizar_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_fur_ilp() {
        let a: Box<dyn ILPEncoder> = Box::new(MehdiNizarOrderEncoder::new());
        let solver = Box::new(Gurobi::new(a));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_mehdi_nizar_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_fur_ilp() {
        let solver = Box::new(Gurobi::new(Box::new(MehdiNizarOrderEncoder::new())));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_mehdi_nizar_prec.txt",
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
            "./bench/results/complete_class_instances_mehdi_nizar_original_optimization.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_original_ilp() {
        let a: Box<dyn ILPEncoder> = Box::new(MehdiNizarOriginalEncoder::new());
        let solver = Box::new(Gurobi::new(a));
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_mehdi_nizar_original_optimization.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_original_ilp() {
        let solver = Box::new(Gurobi::new(Box::new(MehdiNizarOriginalEncoder::new())));
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_mehdi_nizar_optimization.txt",
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
    pub fn complete_test_lawrenko_b_and_b() {
        let solver = Box::new(BranchAndBound::new());
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_branch_and_bound.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_b_and_b() {
        let solver = Box::new(BranchAndBound::new());
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_branch_and_bound.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_compressed_b_and_b() {
        let solver = Box::new(CompressedBnB::new());
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_compressed_branch_and_bound.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_compressed_b_and_b() {
        let solver = Box::new(CompressedBnB::new());
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_compressed_branch_and_bound.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_compressed_b_and_b() {
        let solver = Box::new(CompressedBnB::new());
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_compressed_branch_and_bound.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_compressed_b_and_b_inter() {
        let solver = Box::new(CompressedBnB::new_inter());
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_compressed_branch_and_bound_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_compressed_b_and_b_inter() {
        let solver = Box::new(CompressedBnB::new_inter());
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_compressed_branch_and_bound_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_compressed_b_and_b_inter() {
        let solver = Box::new(CompressedBnB::new_inter());
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_compressed_branch_and_bound_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_compressed_b_and_b_base() {
        let solver = Box::new(CompressedBnB::new_basic());
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_compressed_branch_and_bound_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_compressed_b_and_b_base() {
        let solver = Box::new(CompressedBnB::new_basic());
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_compressed_branch_and_bound_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_compressed_b_and_b_base() {
        let solver = Box::new(CompressedBnB::new_basic());
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_compressed_branch_and_bound_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_hj() {
        let solver = Box::new(HJ::new());
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_hj.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_hj() {
        let solver = Box::new(HJ::new());
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_hj.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_hj() {
        let solver = Box::new(HJ::new());
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_hj.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_hj_inter() {
        let solver = Box::new(HJ::new_inter());
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_hj_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_hj_inter() {
        let solver = Box::new(HJ::new_inter());
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_hj_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_hj_inter() {
        let solver = Box::new(HJ::new_inter());
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_hj_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_hj_base() {
        let solver = Box::new(HJ::new_base());
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_hj_base.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_hj_base() {
        let solver = Box::new(HJ::new_base());
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_hj_base.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_hj_base() {
        let solver = Box::new(HJ::new_base());
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_hj_base.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_class_double() {
        rayon::ThreadPoolBuilder::new()
            .num_threads(40)
            .build_global()
            .unwrap();
        let sat_encoder = Box::new(Precedence::new(Box::new(BddInterComp::new()), 1));
        let unsat_encoder = Box::new(BinmergeSimpEncoder::new());
        let solver = Box::new(MultiSatSolverManager { sat_solver: Box::new(Kissat::new()), unsat_solver: Box::new(Kissat::new()), makespan_scheduler: Box::new(LinearMakespan {}) , sat_encoder, unsat_encoder });
        test_solver(
            solver,
            "./bench/class_instances/",
            "./bench/results/complete_class_instances_multi.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_double() {
        rayon::ThreadPoolBuilder::new()
        .num_threads(40)
        .build_global()
        .unwrap();
        let sat_encoder = Box::new(Precedence::new(Box::new(BddInterComp::new()), 1));
        let unsat_encoder = Box::new(BinmergeSimpEncoder::new());
        let solver = Box::new(MultiSatSolverManager { sat_solver: Box::new(Kissat::new()), unsat_solver: Box::new(Kissat::new()), makespan_scheduler: Box::new(LinearMakespan {}) , sat_encoder, unsat_encoder });
        test_solver(
            solver,
            "./bench/lawrenko/",
            "./bench/results/complete_lawrenko_multi.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_franca_double() {
        rayon::ThreadPoolBuilder::new()
        .num_threads(40)
        .build_global()
        .unwrap();
        let sat_encoder = Box::new(Precedence::new(Box::new(BddInterComp::new()), 1));
        let unsat_encoder = Box::new(BinmergeSimpEncoder::new());
        let solver = Box::new(MultiSatSolverManager { sat_solver: Box::new(Kissat::new()), unsat_solver: Box::new(Kissat::new()), makespan_scheduler: Box::new(LinearMakespan {}) , sat_encoder, unsat_encoder });
        test_solver(
            solver,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/complete_franca_frangioni_multi.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn complete_test_lawrenko_thesis_full() {
        let solver = Box::new(CompressedBnB::new());
        test_solver(
            solver,
            "./bench/lawrenko_thesis/",
            "./bench/results/complete_lawrenko_thesis_bnb.txt",
        )
    }

    

    #[test]
    #[ignore]
    pub fn thesis_tests() {
        // complete_test_class_original_ilp();
        // complete_test_lawrenko_original_ilp();

        //complete_test_class_fur_ilp();
        //complete_test_lawrenko_fur_ilp();

        // complete_test_class_b_and_b();
        // complete_test_lawrenko_b_and_b();

        // complete_test_class_compressed_b_and_b();
        // complete_test_lawrenko_compressed_b_and_b();
        // complete_test_franca_compressed_b_and_b();

        // complete_test_class_compressed_b_and_b_inter();
        // complete_test_lawrenko_compressed_b_and_b_inter();
        // complete_test_franca_compressed_b_and_b_inter();

        // complete_test_class_compressed_b_and_b_base();
        // complete_test_lawrenko_compressed_b_and_b_base();
        // complete_test_franca_compressed_b_and_b_base();

        // complete_test_class_hj_inter();
        // complete_test_franca_hj_inter();
        // complete_test_lawrenko_hj_inter();

        complete_test_class_hj();
        complete_test_lawrenko_hj();
        complete_test_franca_hj();

        // complete_test_class_hj_base();
        // complete_test_lawrenko_hj_base();
        // complete_test_franca_hj_base();

        // complete_test_class_basic();
        // complete_test_lawrenko_basic();
        // complete_test_franca_basic();

        // complete_test_class_binmerge();
        // complete_test_lawrenko_binmerge();
        // complete_test_franca_binmerge();

        // complete_test_class_bdd_native();
        // complete_test_lawrenko_bdd_native();

        // complete_test_class_bdd_prec();
        // complete_test_lawrenko_bdd_prec();

        // complete_test_lawrenko_bdd_inter_only();

        // rayon::ThreadPoolBuilder::new()
        //     .num_threads(10)
        //     .build_global()
        //     .unwrap();

        // complete_test_franca_b_and_b();
        //complete_test_class_bdd_inter_only();
        //complete_test_franca_bdd_native();
        //complete_test_franca_bdd_prec();
        //complete_test_franca_bdd_inter_only();
        //complete_test_franca_original_ilp();
        //complete_test_franca_fur_ilp();
    }

}
