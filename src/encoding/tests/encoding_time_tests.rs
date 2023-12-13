// cargo test -r --test-threads=1 --features encoding_class_instances

#[cfg(test)]
mod tests {
    use crate::{
        bounds::{
            bound::Bound,
            lower_bounds::{max_job_size, middle, pigeon_hole},
            upper_bounds::{lpt, lptp, lptpp},
        },
        common::timeout::Timeout,
        encoding::{
            basic_encoder::BasicEncoder,
            binmerge_native::BinmergeEncoder,
            encoder::{Clause, Encoder},
            pb_bdd_inter::PbInter,
            pb_bdd_native::PbNativeEncoder,
            pb_bdd_pysat::PbPysatEncoder,
        },
        input_output::{self},
        problem_instance::partial_solution::PartialSolution,
        problem_simplification::{
            fill_up_rule, final_simp_rule, half_size_rule, simplification_rule::SimpRule,
        },
    };
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};
    use std::{
        fs::{self, File},
        io::Write,
    };

    fn test_file(encoder: &mut Box<dyn Encoder>, file_name: &String) -> Vec<String> {
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

        let mut out: Vec<String> = vec![];

        for makespan in lower_bound..(upper_bound.makespan + 1).min(lower_bound + 10) {
            print!("a");
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

            let encoding_time: f64 = 60.0;

            let timer = &Timeout::new(encoding_time);
            let success = encoder.basic_encode(&pi, makespan, &timer, 500_000_000);

            if !success {
                continue;
            }

            let encoding_time = encoding_time - timer.remaining_time();

            let clauses = encoder.output();
            let string_clauses = input_output::to_dimacs::to_dimacs(
                clauses,
                encoder.get_num_vars(),
                &Timeout::new(100.0),
            )
            .unwrap();
            out.push(format!(
                "{}_{} {} {} {} {}",
                file_name,
                makespan,
                encoding_time,
                pi.instance.num_jobs,
                pi.instance.num_processors,
                string_clauses.len(),
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
    pub fn test_class_encoding_basic() {
        let mut encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        test_encoder(
            &mut encoder,
            "./bench/class_instances/",
            "./bench/results/encoding_class_instances_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_class_encoding_pysat() {
        let mut a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/encoding_class_instances_pysat.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_class_encoding_bdd_native() {
        let mut a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/encoding_class_instances_bdd.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_class_encoding_inter() {
        let mut a: Box<dyn Encoder> = Box::new(PbInter::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/encoding_class_instances_inter.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_class_encoding_binmerge() {
        let mut a: Box<dyn Encoder> = Box::new(BinmergeEncoder::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/results/encoding_class_instances_binmerge.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_basic_franca_encoding() {
        let mut encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        test_encoder(
            &mut encoder,
            "./bench/franca_frangioni/standardised/",
            "./bench/results/encoding_franca_frangioni_basic.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn tes_franca_encoding_pysat() {
        let mut a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        test_encoder(
            &mut a,
            "./bench/franca_frangioni_standardised/",
            "./bench/results/encoding_franca_frangioni_binmerge.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_franca_encoding_bdd_native() {
        let mut a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        test_encoder(
            &mut a,
            //"./bench/class_instances/",
            //"./bench/class_instances/"
            "./bench/franca_frangioni_standardised/",
            "./bench/results/encoding_franca_frangioni_bdd.txt",
        )
    }

    #[test]
    #[ignore]
    pub fn test_franca_encoding_inter() {
        let mut a: Box<dyn Encoder> = Box::new(PbInter::new());
        test_encoder(
            &mut a,
            //"./bench/class_instances/",
            //"./bench/class_instances/"
            "./bench/franca_frangioni_standardised/",
            "./bench/results/encoding_franca_frangioni_inter.txt",
        )
    }

    // Ok so this isnt something to write home about, but for very large formulas we can use cache efficiency to speed up transfer times
    // These formulas can be like 2GB+ though, before it makes a diff, and those 10 seconds spared are probs not that important
    #[test]
    #[ignore]
    pub fn silly_test() {
        let start = Timeout::new(1000.0);
        let mut clauses: Vec<Clause> = vec![];
        for _ in 0..100000000 {
            clauses.push(Clause {
                vars: vec![1, 2, 3, 4],
            });
        }

        let mut out = String::new();
        for a in clauses {
            for v in a.vars {
                out += &format!("{} ", v);
            }
            out += "0\n"
        }

        print!("regular took {} seconds", 1000.0 - start.remaining_time());
        let start = Timeout::new(1000.0);
        let mut clauses: Vec<i32> = vec![];
        for _ in 0..100000000 {
            clauses.append(&mut vec![1, 2, 3, 4]);
            clauses.push(0);
        }

        let mut out = String::new();
        for a in clauses {
            out += &format!("{} ", a);
        }
        print!("Better took {} seconds", 1000.0 - start.remaining_time());
    }
}
