// cargo test -r --test-threads=1 --features encoding_class_instances

#[cfg(test)]
mod tests {
    use rayon::{
        iter::IntoParallelRefIterator,
        prelude::{IntoParallelIterator, ParallelIterator},
    };

    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{lifting::Lifting, max_job_size, middle, pigeon_hole},
            upper_bounds::{lpt, lptp, lptpp, mss::MSS},
        },
        common::timeout::Timeout,
        encoding::{sat_encoder::Encoder, sat_encoding::{basic_encoder::BasicEncoder, pb_bdd_pysat::PbPysatEncoder, precedence_encoder::Precedence, pb_bdd_native::PbNativeEncoder, binmerge_native::BinmergeEncoder, binmerge_simp::BinmergeSimpEncoder}
        },
        input_output::{self},
        problem_instance::partial_solution::PartialSolution,
        problem_simplification::{
            fill_up_rule::FillUpRule, final_simp_rule::FinalizeRule, half_size_rule::HalfSizeRule,
            simplification_rule::SimpRule,
        },
        solvers::sat_solver::{kissat::Kissat, sat_solver::SatSolver},
    };
    use std::{
        fs::{self, File},
        io::Write,
    };

    fn bound_file(file_name: &String) -> Vec<(String, PartialSolution, usize)> {
        let instance = input_output::from_file::read_from_file(&file_name.to_string());

        let bounds: Vec<Box<dyn Bound>> = vec![
            Box::new(pigeon_hole::PigeonHole {}),
            Box::new(max_job_size::MaxJobSize {}),
            Box::new(middle::MiddleJobs {}),
            Box::new(lpt::LPT {}),
            Box::new(lptp::Lptp {}),
            Box::new(lptpp::Lptpp {}),
            Box::new(Lifting::new_deterministic(1)),
            Box::new(MSS::new_deterministic(4)),
        ];

        let (mut lower_bound, mut upper_bound) = (0, None);
        for bound in bounds {
            (lower_bound, upper_bound) =
                bound.bound(&instance, lower_bound, upper_bound, &Timeout::new(10.0));
        }
        let upper_bound = upper_bound.unwrap();
        let pi = PartialSolution::new(instance);

        let mut out: Vec<(String, PartialSolution, usize)> = vec![];

        for makespan_to_test in lower_bound..upper_bound.makespan {
            let mut hsr = HalfSizeRule {};
            let mut fur: FillUpRule = FillUpRule {};
            let mut finalize: FinalizeRule = FinalizeRule {};
            let pi = hsr.simplify(&pi, makespan_to_test);
            if pi.is_none() {
                continue;
            }
            let pi = pi.unwrap();
            let pi = fur.simplify(&pi, makespan_to_test);
            if pi.is_none() {
                continue;
            }
            let pi = pi.unwrap();
            let pi = finalize.simplify(&pi, makespan_to_test);
            if pi.is_none() {
                continue;
            }
            let pi = pi.unwrap();
            out.push((file_name.clone(), pi, makespan_to_test));
        }

        return out;
    }

    fn solve_instance(
        pi: &PartialSolution,
        makespan_to_test: usize,
        encoder: &mut Box<dyn Encoder>,
        file_name: &String,
    ) -> Option<String> {
        //loop {
        //    let sys = System::new_all();
        //    let available_mem = sys.available_memory();
        //    if available_mem > 50000000000 {
        //        break;
        //    }
        //    thread::sleep(Duration::from_secs(5));
        //}
        println!(
            "solving file {} with makespan {}",
            file_name, makespan_to_test
        );
        let succ = encoder.basic_encode(&pi, makespan_to_test, &Timeout::new(100.0), 500_000_000);
        if !succ {
            return None;
        }

        let solving_time: f64 = 300.0;
        let timer = &Timeout::new(solving_time);
        let res = encoder.output();
        let num_vars = encoder.get_num_vars();
        let len = res.len();

        let mut solver = Kissat::new();

        let sol = solver.solve(res, num_vars, &timer);

        let solving_time = solving_time - timer.remaining_time();
        if sol.is_timeout() {
            return None;
        }

        let is_sat = if sol.is_sat() { 1 } else { 0 };
        if sol.is_sat() {
            let solution = encoder.decode(&pi.instance, &sol.unwrap().as_ref().unwrap());
            assert!(solution.makespan <= makespan_to_test);
        }
        return Some(format!(
            "{}_{} {} {} {} {} {}",
            file_name,
            makespan_to_test,
            solving_time,
            is_sat,
            pi.instance.num_jobs,
            pi.instance.num_processors,
            len,
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
            .collect();

        let instances: Vec<(String, PartialSolution, usize)> = files
            .par_iter()
            .map(|x| bound_file(x))
            .flat_map(|x| x)
            .collect();
        println!("testing {} instances", instances.len());

        let instances_with_encoder: Vec<(String, PartialSolution, usize, Box<dyn Encoder>)> =
            instances
                .into_iter()
                .map(|(x, y, z)| (x, y, z, encoder.clone()))
                .collect();

        let result: Vec<String> = instances_with_encoder
            .into_par_iter()
            .map(|(file_name, pi, makespan_to_test, mut encoder)| {
                solve_instance(&pi, makespan_to_test, &mut encoder, &file_name)
            })
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

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
    pub fn test_solve_time_class_basic() {
        let mut encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        test_encoder(
            &mut encoder,
            "./bench/class_instances/",
            "./bench/results/class_instances_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_class_pysat_alone() {
        let mut a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_pysat.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_class_pysat_with_precedence() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbPysatEncoder::new()), 2));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_pysat_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_class_bdd_native() {
        let mut a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_bdd.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_class_bdd_native_with_precedence() {
        let mut a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(PbNativeEncoder::new()), 2));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_bdd_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_class_binmerge_native() {
        let mut a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BinmergeEncoder::new()), 1));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_binmerge_prec_1.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_class_binsimp() {
        let mut a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(BinmergeSimpEncoder::new()), 1));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_binsimp_prec_1.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_basic() {
        let mut encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        test_encoder(
            &mut encoder,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_pysat() {
        let mut a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_binmerge.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_pysat_with_precedence() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbPysatEncoder::new()), 2));
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_binmerge_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_pysat_with_precedence_1() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbPysatEncoder::new()), 1));
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_binmerge_prec_1.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_bdd_native() {
        let mut a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_bdd.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_bdd_native_with_precedence() {
        let mut a: Box<dyn Encoder> =
            Box::new(Precedence::new(Box::new(PbNativeEncoder::new()), 2));
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_bdd_prec.txt",
        )
    }
}
