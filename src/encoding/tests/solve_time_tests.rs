// cargo test -r --test-threads=1 --features encoding_class_instances

#[cfg(test)]
mod tests {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};
    use sysinfo::{System, SystemExt};

    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{max_job_size, middle, pigeon_hole},
            upper_bounds::{lpt, lptp, lptpp},
        },
        common::timeout::Timeout,
        encoding::{
            basic_encoder::BasicEncoder, basic_with_precedence::Precedence, encoder::Encoder,
            pb_bdd_inter::PbInter, pb_bdd_inter_better::PbInterDyn, pb_bdd_native::PbNativeEncoder,
            pb_bdd_pysat::PbPysatEncoder,
        },
        input_output::{self},
        problem_instance::partial_solution::PartialSolution,
        problem_simplification::{
            fill_up_rule, final_simp_rule, half_size_rule, simplification_rule::SimpRule,
        },
        solvers::{sat_solver::kissat::Kissat, solver::SatSolver},
    };
    use std::{
        fs::{self, File},
        io::Write,
        thread,
        time::Duration,
    };

    fn test_file(
        encoder: &mut Box<dyn Encoder>,
        file_name: &String,
    ) -> Vec<String> {
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
        let upper_bound = upper_bound.unwrap();
        let pi = PartialSolution::new(instance);
        let mut solver = Kissat::new();

        let mut out: Vec<String> = vec![];

        println!("solving file {}", file_name);
        //let mut makespans_to_check = vec![lower_bound, lower_bound - 5, lower_bound - 10];
        //
        //if lower_bound != upper_bound.makespan {
        //    makespans_to_check.push(upper_bound.makespan);
        //}
        //makespans_to_check.push(upper_bound.makespan + 5);
        //makespans_to_check.push(upper_bound.makespan + 10);
        let makespans_to_check = lower_bound..upper_bound.makespan + 1;
        for makespan in makespans_to_check {
            let mut hsr = half_size_rule::HalfSizeRule {};
            let mut fur = fill_up_rule::FillUpRule {};
            let mut finalize: final_simp_rule::FinalizeRule = final_simp_rule::FinalizeRule {};
            let pi = hsr.simplify(&pi, makespan);
            if pi.is_none() {
                continue;
            }
            let pi = fur.simplify(pi.as_ref().unwrap(), makespan);
            if pi.is_none() {
                continue;
            }
            let pi = finalize.simplify(pi.as_ref().unwrap(), makespan);
            if pi.is_none() {
                continue;
            }
            let pi: PartialSolution = pi.unwrap();



            loop {
                let sys = System::new_all();
                let available_mem = sys.available_memory();
                if available_mem > 50000000000 {
                    break;
                }
                thread::sleep(Duration::from_secs(5));
            }
            let succ = encoder.basic_encode(&pi, makespan, &Timeout::new(100.0), 500_000_000);
            if !succ {
                continue;
            }

            let solving_time: f64 = 300.0;
            let timer = &Timeout::new(solving_time);
            let res = encoder.output();
            let num_vars = encoder.get_num_vars();
            let len = res.len();

            let sol = solver.solve(res, num_vars, &timer);

            let solving_time = solving_time - timer.remaining_time();
            if sol.is_timeout() {
                continue;
            }
            out.push(format!(
                "{}_{} {} {} {} {} {}",
                file_name,
                makespan,
                solving_time,
                if sol.is_sat() { 1 } else { 0 },
                pi.instance.num_jobs,
                pi.instance.num_processors,
                len,
            ))
        }
        return out;
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

        let files: Vec<(String, Box<dyn Encoder>)> = files
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
    pub fn test_solve_time_class_inter_with_precedence_unopt() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::_new_unopt()), 2));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_inter_prec_unopt.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_class_inter_with_precedence_opt() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::new()), 2));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_inter_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_class_inter_precedence_1() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::new()), 1));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_inter_prec_1.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_class_interp_with_precedence() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInterDyn::new()), 2));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/class_instances_inter+_prec.txt",
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

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_inter_with_precedence_unopt() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::_new_unopt()), 2));
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_inter_prec_unopt.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_inter_with_precedence() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::new()), 2));
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_inter_prec.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_inter_precedence_1() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::new()), 1));
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_inter_prec_1.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_inter() {
        let mut a: Box<dyn Encoder> = Box::new(PbInter::new());
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_solve_time_franca_interp_with_precedence() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInterDyn::new()), 2));
        test_encoder(
            &mut a,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/franca_frangioni_inter+_prec.txt",
        )
    }

}
