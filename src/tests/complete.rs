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
        },
        common::timeout::Timeout,
        encoding::ilp_encoding::mehdi_nizar_prec::MehdiNizarOrderEncoder,
        input_output,
        solvers::{
            branch_and_bound::{compressed_bnb::CompressedBnB, hj::HJ}, cdsm::cdsm::CDSM,
            ilp_solver::gurobi::Gurobi, solver_manager::SolverManager,
        },
    };
    use std::{
        fs::{self, File},
        io::Write,
        sync::{Arc, Mutex},
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
        let total_timeout_f64: f64 = 500.0;
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
            Box::new(Lifting::new_deterministic(1)),
            Box::new(mss::MSS::new_deterministic(4)),
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
        // println!("{}", file_name);

        let sol = solver.solve(&instance, lower_bound, &upper_bound, &total_timeout, false);
        if sol.is_none() {
            println!("could not solve file {}", file_name);
            assert!(total_timeout.time_finished());
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
            // .into_iter()
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

    const FOLDERS_TO_TEST: [&'static str; 11] = [
        "./bench/class_instances/",
        "./bench/franca_frangioni/standardised/",
        "./bench/lawrenko/",
        "./bench/cnf/",
        "./bench/graph/",
        "./bench/huebner/",
        "./bench/laupichler/",
        "./bench/lehmann/",
        "./bench/planted/",
        "./bench/sc2022",
        "./bench/schreiber/"
    ];
    const BENCHMARK_NAMES: [&'static str; 11] = [
        "berndt",
        "franca_frangioni",
        "lawrenko",
        "real_cnf",
        "real_graph",
        "real_rt_huebner",
        "real_rt_anni",
        "real_rt_lehmann",
        "real_planted",
        "real_sc2022",
        "real_rt_schreiber",
    ];

    #[test]
    #[ignore]
    pub fn complete_test_base_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            false,
            false,
            false,
            false,
            false,
            1_000_000_000,
        ));
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/complete_{}_cdsm_base.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn complete_test_last_size_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            false,
            false,
            false,
            true,
            false,
            1_000_000_000,
        ));
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/complete_{}_cdsm_last_size.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn complete_test_inter_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            true,
            false,
            false,
            true,
            false,
            1_000_000_000,
        ));
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/complete_{}_cdsm_inter.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn complete_test_fur_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            true,
            true,
            false,
            true,
            false,
            1_000_000_000,
        ));
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/complete_{}_cdsm_fur.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn complete_test_irrelevance_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            true,
            true,
            true,
            true,
            false,
            1_000_000_000,
        ));
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/complete_{}_cdsm_irrelevance.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn complete_test_cdsm() {
        let solver = Box::new(CDSM::new_with_rules(
            true,
            true,
            true,
            true,
            true,
            1_000_000_000,
        ));
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/complete_{}_cdsm.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn complete_test_ilp() {
        let solver = Box::new(Gurobi::new(Box::new(MehdiNizarOrderEncoder::new())));
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/complete_{}_ilp.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn complete_test_hj() {
        let solver = Box::new(HJ::new_base());
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/complete_{}_hj.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }


}
