#[cfg(test)]
mod tests {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};

    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{
                lifting::Lifting,
                lifting_weak::LiftingWeak,
                max_job_size::{self},
                middle::{self},
                pigeon_hole::{self, PigeonHole},
                sss_bound_tightening::{self, SSSBoundStrengthening},
            },
            upper_bounds::{
                lpt::{self, LPT},
                lptp::{self, Lptp},
                lptpp::{self, Lptpp},
                mss::MSS,
            },
        },
        common::timeout::Timeout,
        input_output,
    };
    use std::{
        fs::{self, File},
        io::Write,
    };

    fn test_file_bound(bound_to_test: Box<dyn Bound>, file_name: &String) -> Option<String> {
        let instance = input_output::from_file::read_from_file(file_name);
        let total_timeout_f64: f64 = 10.0;

        // --------------CALCULATING BOUNDS--------------
        let bounds: Vec<Box<dyn Bound>> = vec![
            Box::new(pigeon_hole::PigeonHole {}),
            Box::new(max_job_size::MaxJobSize {}),
            Box::new(middle::MiddleJobs {}),
            Box::new(lpt::LPT {}),
            Box::new(lptp::Lptp {}),
            Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
            Box::new(lptpp::Lptpp {}),
            // Box::new(lifting::Lifting::new()),
            // Box::new(MSS::new()),
        ];

        let (mut lower_bound, mut upper_bound) = (1, None);
        for i in 0..bounds.len() {
            let precomp_timeout = Timeout::new(10.0);
            let bound = &bounds[i];
            (lower_bound, upper_bound) =
                bound.bound(&instance, lower_bound, upper_bound, &precomp_timeout);
        }
        let upper_bound = upper_bound.unwrap();

        //--------------SOLVING---------------------------

        let total_timeout = Timeout::new(10.0);
        let (new_lower, new_upper) =
            bound_to_test.bound(&instance, lower_bound, Some(upper_bound), &total_timeout);
        let new_upper = new_upper.unwrap();

        return Some(format!(
            "{} {} {} {} {} {}",
            file_name,
            total_timeout_f64 - total_timeout.remaining_time(),
            new_lower as f64,
            new_upper.makespan as f64,
            instance.num_processors,
            (instance.num_jobs as f64) / (instance.num_processors as f64),
        ));
    }

    fn test_bound(solver: Box<dyn Bound>, in_dirname: &str, out_dirname: &str) {
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

        let files: Vec<(String, Box<dyn Bound>)> = files
            .iter()
            .map(|x| (x.clone(), solver.clone()))
            .collect::<Vec<_>>();
        let result = files
            .into_par_iter()
            //.into_iter()
            .map(|(path, encoder)| test_file_bound(encoder, &path))
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
        "/global_data/pcmax_instances/finaler/cnf/",
        "/global_data/pcmax_instances/finaler/graph/",
        "/global_data/pcmax_instances/finaler/planted/",
        "/global_data/pcmax_instances/finaler/anni/",
        "/global_data/pcmax_instances/finaler/huebner/",
        "/global_data/pcmax_instances/finaler/lehmann/",
        "/global_data/pcmax_instances/finaler/schreiber/",
        "/global_data/pcmax_instances/finaler/laupichler/",
    ];
    const BENCHMARK_NAMES: [&'static str; 11] = [
        "berndt",
        "franca_frangioni",
        "lawrenko",
        "real_cnf",
        "real_graph",
        "real_planted",
        "real_rt_anni",
        "real_rt_huebner",
        "real_rt_lehmann",
        "real_rt_schreiber",
        "real_rt_laupichler",
    ];

    #[test]
    #[ignore]
    pub fn test_bound_trivial() {
        let solver = Box::new(PigeonHole {});
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/bound_{}_trivial.txt", BENCHMARK_NAMES[i]);
            test_bound(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn test_bound_lpt() {
        let solver = Box::new(LPT {});
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/bound_{}_lpt.txt", BENCHMARK_NAMES[i]);
            test_bound(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn test_bound_lptp() {
        let solver = Box::new(Lptp {});
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/bound_{}_lptp.txt", BENCHMARK_NAMES[i]);
            test_bound(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn test_bound_lptpp() {
        let solver = Box::new(Lptpp {});
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/bound_{}_lptpp.txt", BENCHMARK_NAMES[i]);
            test_bound(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn test_bound_sss_bound_strengthening() {
        let solver = Box::new(SSSBoundStrengthening {});
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!(
                "./bench/results/bound_{}_sss_bound_strengthening.txt",
                BENCHMARK_NAMES[i]
            );
            test_bound(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn test_bound_lifting() {
        let solver = Box::new(Lifting::new());
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/bound_{}_lifting.txt", BENCHMARK_NAMES[i]);
            test_bound(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn test_bound_mss() {
        let solver = Box::new(MSS::new());
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/bound_{}_mss.txt", BENCHMARK_NAMES[i]);
            test_bound(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn test_bound_lifting_weak() {
        let solver = Box::new(LiftingWeak::new());
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!(
                "./bench/results/bound_{}_lifting_weak.txt",
                BENCHMARK_NAMES[i]
            );
            test_bound(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }

    #[test]
    #[ignore]
    pub fn test_bound_ssss() {
        let solver = Box::new(MSS::new());
        for i in 0..FOLDERS_TO_TEST.len() {
            let out_file_name = format!("./bench/results/bound_{}_ssss.txt", BENCHMARK_NAMES[i]);
            test_bound(solver.clone(), FOLDERS_TO_TEST[i], &out_file_name);
        }
    }
}
