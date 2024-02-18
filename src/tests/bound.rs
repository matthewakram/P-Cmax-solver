#[cfg(test)]
mod tests {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};

    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{lifting::{self, Lifting}, lifting_weak::LiftingWeak, max_job_size::{self, MaxJobSize}, middle::{self, MiddleJobs}, pigeon_hole, sss_bound_tightening},
            upper_bounds::{lpt::{self, LPT}, lptp::{self, Lptp}, lptpp::{self, Lptpp}, mss::MSS},
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
            Box::new(lifting::Lifting::new_deterministic(1)),
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
        let (new_lower, new_upper) = bound_to_test.bound(&instance, lower_bound, Some(upper_bound), &total_timeout);
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
            .map(|(path, encoder)| {
                test_file_bound(encoder, &path)
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
    pub fn bound_class_pigeon() {
        let bound = Box::new(pigeon_hole::PigeonHole{});
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_pigeon.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_pigeon() {
        let bound = Box::new(pigeon_hole::PigeonHole{});
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_pigeon.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_pigeon() {
        let bound = Box::new(pigeon_hole::PigeonHole{});
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_pigeon.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_class_max() {
        let bound = Box::new(MaxJobSize{});
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_max_job_size.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_max() {
        let bound = Box::new(MaxJobSize{});
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_max_job_size.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_max() {
        let bound = Box::new(MaxJobSize{});
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_max_job_size.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_class_middle_jobs() {
        let bound = Box::new(MiddleJobs{});
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_middle_jobs.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_middle_jobs() {
        let bound = Box::new(MiddleJobs{});
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_middle_jobs.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_middle_jobs() {
        let bound = Box::new(MiddleJobs{});
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_middle_jobs.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_class_lpt() {
        let bound = Box::new(LPT{});
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_lpt.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_lpt() {
        let bound = Box::new(LPT{});
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_lpt.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_lpt() {
        let bound = Box::new(LPT{});
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_lpt.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_class_lptp() {
        let bound = Box::new(Lptp{});
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_lptp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_lptp() {
        let bound = Box::new(Lptp{});
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_lptp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_lptp() {
        let bound = Box::new(Lptp{});
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_lptp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_class_lptpp() {
        let bound = Box::new(Lptpp{});
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_lptpp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_lptpp() {
        let bound = Box::new(Lptpp{});
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_lptpp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_lptpp() {
        let bound = Box::new(Lptpp{});
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_lptpp.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_class_sss_bound_strengthening() {
        let bound = Box::new(SssBoundStrengthTester{});
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_sss_bound_strengthening.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_sss_bound_strengthening() {
        let bound = Box::new(SssBoundStrengthTester{});
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_sss_bound_strengthening.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_sss_bound_strengthening() {
        let bound = Box::new(SssBoundStrengthTester{});
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_sss_bound_strengthening.txt",
        )
    }

    #[derive(Clone)]
    struct SssBoundStrengthTester{
    }

    impl Bound for SssBoundStrengthTester{
        fn bound(
            &self,
            problem: &crate::problem_instance::problem_instance::ProblemInstance,
            lower_bound: usize,
            upper_bound: Option<crate::problem_instance::solution::Solution>,
            timeout: &Timeout,
        ) -> (usize, Option<crate::problem_instance::solution::Solution>) {
            let bounds: Vec<Box<dyn Bound>> = vec![
                Box::new(pigeon_hole::PigeonHole {}),
                Box::new(max_job_size::MaxJobSize {}),
                Box::new(middle::MiddleJobs {}),
                Box::new(lpt::LPT {}),
                Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
            ];
            let (mut lower_bound, mut upper_bound) = (lower_bound, upper_bound);

            for i in 0..bounds.len() {
            let bound = &bounds[i];
            (lower_bound, upper_bound) =
                bound.bound(&problem, lower_bound, upper_bound, timeout);
            if timeout.time_finished()
                || (upper_bound.is_some() && upper_bound.as_ref().unwrap().makespan == lower_bound)
            {
                break;
            }
        }
        return (lower_bound, upper_bound);
        }
    }

    #[test]
    #[ignore]
    pub fn bound_class_lifting() {
        let bound = Box::new(Lifting::new());
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_lifting.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_lifting() {
        let bound = Box::new(Lifting::new());
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_lifting.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_lifting() {
        let bound = Box::new(Lifting::new());
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_lifting.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_class_mss() {
        let bound = Box::new(MSS::new());
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_mss.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_mss() {
        let bound = Box::new(MSS::new());
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_mss.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_mss() {
        let bound = Box::new(MSS::new());
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_mss.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_class_lifting_weak() {
        let bound = Box::new(LiftingWeak::new());
        test_bound(
            bound,
            "./bench/class_instances/",
            "./bench/results/bound_class_lifting_weak.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_franca_lifting_weak() {
        let bound = Box::new(LiftingWeak::new());
        test_bound(
            bound,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/bound_franca_lifting_weak.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn bound_lawrenko_lifting_weak() {
        let bound = Box::new(LiftingWeak::new());
        test_bound(
            bound,
            "./bench/lawrenko/",
            "./bench/results/bound_lawrenko_lifting_weak.txt",
        )
    }


}