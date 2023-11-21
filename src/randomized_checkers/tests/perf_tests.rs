#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
    };

    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{max_job_size, middle, pigeon_hole},
            upper_bounds::{lpt, lptp, lptpp},
        },
        common::timeout::Timeout,
        input_output,
        problem_instance::partial_solution::PartialSolution,
        problem_simplification::{
            fill_up_rule, final_simp_rule, half_size_rule, simplification_rule::SimpRule,
        },
        randomized_checkers::{
            bin_search_checker::BinSearchChecker,
            descending_multi_sss_randomized_checker::{self, DescendingMultiSSSRandomizedChecker},
            randomized_checker::{self, RandomizedChecker},
            randomized_multi_sss_randomized_checker::RandomizedMultiSSSRandomizedChecker, bin_search_job_assignment::BinSearchJobAssignmentChecker, bin_search_ordered_job_assignment::BinSearchOrderedJobAssignmentChecker,
        },
        solvers::sat_solver::kissat::Kissat,
    };

    fn test_file(checker: &mut Box<dyn RandomizedChecker>, file_name: &String) -> Vec<String> {
        let instance = input_output::from_file::read_from_file(&file_name.to_string());

        let bounds: Vec<Box<dyn Bound>> = vec![
            Box::new(pigeon_hole::PigeonHole {}),
            Box::new(max_job_size::MaxJobSize {}),
            Box::new(middle::MiddleJobs {}),
            Box::new(lpt::LPT {}),
            Box::new(lptp::Lptp {}),
            Box::new(lptpp::Lptpp {}),
        ];

        let (mut lower_bound, mut upper_bound) = (0, None);
        for bound in bounds {
            (lower_bound, upper_bound) =
                bound.bound(&instance, lower_bound, upper_bound, &Timeout::new(1000.0));
        }
        let mut upper_bound = upper_bound.unwrap();
        let pi = PartialSolution::new(instance);

        let mut out: Vec<String> = vec![];

        let mut hsr = half_size_rule::HalfSizeRule {};
        let mut fur = fill_up_rule::FillUpRule {};
        let mut finalize: final_simp_rule::FinalizeRule = final_simp_rule::FinalizeRule {};

        let solving_time: f64 = 60.0;
        let timer = &Timeout::new(solving_time);
        println!("solving file name {}", file_name);
        let original_upper_bound = upper_bound.makespan;

        while lower_bound != upper_bound.makespan {
            let pi = hsr.simplify(&pi, upper_bound.makespan);
            if pi.is_none() {
                return vec![];
            }
            let pi = fur.simplify(pi.as_ref().unwrap(), upper_bound.makespan);
            if pi.is_none() {
                return vec![];
            }
            let pi = finalize.simplify(pi.as_ref().unwrap(), upper_bound.makespan);
            if pi.is_none() {
                return vec![];
            }
            let pi: PartialSolution = pi.unwrap();

            let sol = checker.is_sat(&pi, upper_bound.makespan - 1, &timer);
            if timer.time_finished() {
                break;
            }
            if sol.is_none() {
                continue;
            }
            if sol.is_some() {
                upper_bound = sol.unwrap();
            }
        }

        if upper_bound.makespan != original_upper_bound {
            out.push(format!(
                "{} {} {} {}",
                file_name, upper_bound.makespan, pi.instance.num_jobs, pi.instance.num_processors,
            ));
        }

        return out;
    }

    fn test_encoder(encoder: Box<dyn RandomizedChecker>, in_dirname: &str, out_dirname: &str) {
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
            .collect();

        let files: Vec<(String, Box<dyn RandomizedChecker>)> = files
            .iter()
            .map(|x| (x.clone(), encoder.clone()))
            .collect::<Vec<_>>();
        let result = files
            .into_par_iter()
            //.into_iter()
            .map(|(path, mut encoder)| test_file(&mut encoder, &path))
            .flat_map(|s| s)
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
    pub fn checker_class_test_descending() {
        let checker = Box::new(DescendingMultiSSSRandomizedChecker {});
        test_encoder(
            checker,
            "./bench/class_instances",
            "./bench/results/checker_descending.txt",
        );
    }

    #[test]
    #[ignore]
    pub fn checker_class_test_random() {
        let checker = Box::new(RandomizedMultiSSSRandomizedChecker {});
        test_encoder(
            checker,
            "./bench/class_instances",
            "./bench/results/checker_random.txt",
        );
    }

    #[test]
    #[ignore]
    pub fn checker_class_test_binary() {
        let checker = Box::new(BinSearchChecker {});
        test_encoder(
            checker,
            "./bench/class_instances",
            "./bench/results/checker_bin_search.txt",
        );
    }

    #[test]
    #[ignore]
    pub fn checker_franca_test_descending() {
        let checker = Box::new(DescendingMultiSSSRandomizedChecker {});
        test_encoder(
            checker,
            "./bench/franca_frangioni/standardised",
            "./bench/results/checker_descending.txt",
        );
    }

    #[test]
    #[ignore]
    pub fn checker_franca_test_random() {
        let checker = Box::new(RandomizedMultiSSSRandomizedChecker {});
        test_encoder(
            checker,
            "./bench/franca_frangioni/standardised",
            "./bench/results/checker_random.txt",
        );
    }

    #[test]
    #[ignore]
    pub fn checker_franca_test_binary() {
        let checker = Box::new(BinSearchChecker {});
        test_encoder(
            checker,
            "./bench/franca_frangioni/standardised",
            "./bench/results/checker_bin_search.txt",
        );
    }

    #[test]
    #[ignore]
    pub fn checker_franca_test_binary_jobs() {
        let checker = Box::new(BinSearchJobAssignmentChecker {});
        test_encoder(
            checker,
            "./bench/franca_frangioni/standardised",
            "./bench/results/checker_bin_search_jobs.txt",
        );
    }

    #[test]
    #[ignore]
    pub fn checker_franca_test_binary_jobs_ordered() {
        let checker = Box::new(BinSearchOrderedJobAssignmentChecker {});
        test_encoder(
            checker,
            "./bench/franca_frangioni/standardised",
            "./bench/results/checker_bin_search_jobs_ordered.txt",
        );
    }
}
