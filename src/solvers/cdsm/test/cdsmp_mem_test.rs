#[cfg(test)]
mod tests {

    use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{max_job_size, middle, pigeon_hole},
            upper_bounds::lpt,
        },
        common::timeout::Timeout,
        input_output,
        solvers::{
            cdsm::{cdsm::CDSM, cdsmp::CDSMP}, solver_manager::SolverManager
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
        // println!("{}", file_name);
        let instance = input_output::from_file::read_from_file(file_name);
        let total_timeout_f64: f64 = 20.0;
        let precomputation_timeout = 10.0;

        // --------------CALCULATING BOUNDS--------------

        let bounds: Vec<Box<dyn Bound>> = vec![
            Box::new(pigeon_hole::PigeonHole {}),
            Box::new(max_job_size::MaxJobSize {}),
            Box::new(middle::MiddleJobs {}),
            Box::new(lpt::LPT {}),
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

    const FOLDERS_TO_TEST: [&'static str;3] = ["./bench/lawrenko/", "/global_data/pcmax_instances/cnf/", "/global_data/pcmax_instances/running-times/sat/"];
    const BENCHMARK_NAMES: [&'static str; 3] = ["lawrenko", "real_sat", "real_runtime"];

    #[test]
    #[ignore]
    pub fn memtest_cdsm_vs_cdsmp(){
        let solver = Box::new(CDSM::new());
        for i in 0..FOLDERS_TO_TEST.len(){
            let out_file_name = format!("./bench/results/memtest_cdsm_{}.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }

        let solver = Box::new(CDSMP::new_with_rules(true, true, true, true, true, 1_048_576));
        for i in 0..FOLDERS_TO_TEST.len(){
            let out_file_name = format!("./bench/results/memtest_cdsmp_{}_1MB.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }

        let solver = Box::new(CDSMP::new_with_rules(true, true, true, true, true, 1_048_5760));
        for i in 0..FOLDERS_TO_TEST.len(){
            let out_file_name = format!("./bench/results/memtest_cdsmp_{}_10MB.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }

        let solver = Box::new(CDSMP::new_with_rules(true, true, true, true, true, 1_048_57600));
        for i in 0..FOLDERS_TO_TEST.len(){
            let out_file_name = format!("./bench/results/memtest_cdsmp_{}_100MB.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }

        let solver = Box::new(CDSMP::new_with_rules(true, true, true, true, true, 1073741824));
        for i in 0..FOLDERS_TO_TEST.len(){
            let out_file_name = format!("./bench/results/memtest_cdsmp_{}_1GB.txt", BENCHMARK_NAMES[i]);
            test_solver(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }


}