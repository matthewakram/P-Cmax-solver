// cargo test -r --test-threads=1 --features encoding_class_instances

#[cfg(test)]
mod tests {
    use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{
                lifting::Lifting,
                max_job_size, middle, pigeon_hole, sss_bound_tightening,
            },
            upper_bounds::{lpt, lptp, lptpp, mss},
        }, common::timeout::Timeout, encoding::{ilp_encoding::mehdi_nizar_prec::MehdiNizarOrderEncoder, sat_encoding::{bdd_inter_comp, pb_bdd_native::{self, PbNativeEncoder}}}, input_output, makespan_scheduling::linear_makespan::LinearMakespan, solvers::{
            branch_and_bound::hj::HJ, cdsm::cdsm::CDSM, ilp_solver::gurobi::Gurobi, sat_solver::{kissat::Kissat, sat_solver_manager::SatSolverManager}, solver_manager::SolverManager
        }
    };
    use std::{
        fmt::format, fs::{self, File, OpenOptions}, io::Write, sync::{Arc, Mutex}
    };

    fn test_file_solver(
        mut solver: Box<dyn SolverManager>,
        file_name: &String,
        progress: Arc<Mutex<usize>>,
        num_total_instances: usize,
        file: Arc<Mutex<File>>
    ) {
        {
            let mut p = progress.lock().unwrap();
            *p += 1;
            println!("solving {}/{}", *p, num_total_instances);
        }

        let instance = input_output::from_file::read_from_file(file_name);
        let total_timeout_f64: f64 = 30.0;
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
            Box::new(Lifting::new_deterministic(4)),
            Box::new(mss::MSS::new_deterministic(1)),
        ];

        let (mut lower_bound, mut upper_bound) = (0, None);

        for i in 0..bounds.len() {
            let precomp_timeout = Timeout::new(precomputation_timeout);
            let bound = &bounds[i];
            (lower_bound, upper_bound) =
                bound.bound(&instance, lower_bound, upper_bound, &precomp_timeout);
            // println!("lower bound is: {}", lower_bound);
            if (upper_bound.is_some() && upper_bound.as_ref().unwrap().makespan == lower_bound)
            {
                break;
            }
        }
        let upper_bound = upper_bound.unwrap();
        let total_timeout = Timeout::new(total_timeout_f64);
        // println!("done with bounds {}", file_name);

        // -------------CHECKING IF SOLUTION HAS BEEN FOUND-----------
        // We maintain that the solution is within [lower_bound, upper_bound]. Note that this is inclusive.

        assert!(lower_bound <= upper_bound.makespan);
        if lower_bound == upper_bound.makespan {
            
            {
                let mut lock = file.lock().unwrap();
                lock.write_all(format!(
                    "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}\n",
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
                ).as_bytes()).unwrap();
            }
            let _ = fs::remove_file(file_name);
            return;
        }
        //--------------SOLVING---------------------------
        // println!("{}", file_name);

        let sol = solver.solve(&instance, lower_bound, &upper_bound, &total_timeout, false);
        if sol.is_none() {
            println!("could not solve file {}", file_name);
            if !total_timeout.time_finished() {
                println!("mem out: {}", file_name);
            }
            //let _ = fs::remove_file(file_name);
            //return;
        }
        //println!("solved file {}", file_name);

        let stats = solver.get_stats();
        let _ = fs::remove_file(file_name);
        {
            let mut lock = file.lock().unwrap();
            lock.write_all(format!(
                "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}\n",
                file_name,
                total_timeout_f64 - total_timeout.remaining_time(),
                if sol.is_none() {-1.0 } else {sol.as_ref().unwrap().makespan as f64},
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
            ).as_bytes()).unwrap();
        }
        let _ = fs::remove_file(file_name);
        return;
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

        let progress: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let num_instances = files.len();
        let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(out_dirname)
        .unwrap();
        let file : Arc<Mutex<File>> = Arc::new(Mutex::new(file));
        files
            .into_par_iter()
            // .into_iter()
            .enumerate()
            .for_each(|(_file_num, (path, encoder))| {
                test_file_solver(encoder, &path, progress.clone(), num_instances, file.clone())
            });
    }


    #[test]
    #[ignore]
    pub fn cumulative_test_base_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            false,
            false,
            false,
            false,
            false,
            1_000_000_000,
        ));
        let out_file_name = format!("./bench/results/complete_cdsm_base.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_last_size_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            false,
            false,
            false,
            true,
            false,
            1_000_000_000,
        ));
        let out_file_name = format!("./bench/results/complete_cdsm_last_size.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_inter_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            true,
            false,
            false,
            true,
            false,
            1_000_000_000,
        ));
        let out_file_name = format!("./bench/results/complete_cdsm_inter.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_fur_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            true,
            true,
            false,
            true,
            false,
            1_000_000_000,
        ));
        let out_file_name = format!("./bench/results/complete_cdsm_fur.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_irrelevance_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            true,
            true,
            true,
            true,
            false,
            1_000_000_000,
        ));
        let out_file_name = format!("./bench/results/complete_cdsm_irrelevance.txt");
        test_solver(solver.clone(), "./bench/cumulative_alt/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            true,
            true,
            false,
            true,
            true,
            1_000_000_000,
        ));
        let out_file_name = format!("./bench/results/complete_cdsm.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_ilp() {
        let solver = Box::new(Gurobi::new(Box::new(MehdiNizarOrderEncoder::new())));
        let out_file_name = format!("./bench/results/complete_ilp.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_original_ilp() {
        let solver = Box::new(Gurobi::new(Box::new(MehdiNizarOrderEncoder::new_original())));
        let out_file_name = format!("./bench/results/complete_ilp_original.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_sat() {
        let solver = Box::new(SatSolverManager::new(Box::new(Kissat::new()), Box::new(LinearMakespan {}) ,Box::new(bdd_inter_comp::BddInterComp::new())));
        let out_file_name = format!("./bench/results/complete_sat.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_basic_sat() {
        let solver = Box::new(SatSolverManager::new(Box::new(Kissat::new()), Box::new(LinearMakespan {}) ,Box::new(pb_bdd_native::PbNativeEncoder::new())));
        let out_file_name = format!("./bench/results/complete_sat.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }


    #[test]
    #[ignore]
    pub fn cumulative_test_silly_test() {
        let solver = Box::new(SatSolverManager::new(Box::new(Kissat::new()), Box::new(LinearMakespan {}) ,Box::new(pb_bdd_native::PbNativeEncoder::new())));
        let out_file_name = format!("./bench/results/silly_test.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }

    #[test]
    #[ignore]
    pub fn cumulative_test_hj() {
        let solver = Box::new(HJ::new_base());
        let out_file_name = format!("./bench/results/complete_hj.txt");
        test_solver(solver.clone(), "./bench/cumulative/", &out_file_name);
    }



}
