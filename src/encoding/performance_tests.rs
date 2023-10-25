#[cfg(test)]
mod tests {
    use crate::{
        bounds::{
            lower_bounds::{
                pigeon_hole,
                sss_bound_tightening::{self}, max_job_size, middle,
            },
            upper_bounds::{lpt, lptp, lptpp, self}, bound::Bound,
        },
        encoding::{
            basic_encoder::BasicEncoder,
            basic_with_precedence::Precedence,
            encoder::{Encoder, OneHotEncoder},
            pb_bdd_native::PbNativeEncoder,
            pb_bdd_pysat::PbPysatEncoder,
            random_encoder::RandomEncoder,
        },
        input_output::{self},
        problem_instance::partial_solution::PartialSolution,
    };
    use std::fs;

    fn test_encoder(encoder: &mut Box<dyn Encoder>, in_dirname: &str, out_dirname: &str) {
        //
        let paths = fs::read_dir(in_dirname).unwrap();
        let mut file_num = 0;
        for path in paths {
            if path
                .as_ref()
                .unwrap()
                .path()
                .display()
                .to_string()
                .ends_with(".txt")
            {
                let instance = input_output::from_file::read_from_file(
                    &path.unwrap().path().display().to_string(),
                );

                let bounds: Vec<Box<dyn Bound>> = vec![
                    Box::new(pigeon_hole::PigeonHole {}),
                    Box::new(max_job_size::MaxJobSize {}),
                    Box::new(middle::MiddleJobs {}),
                    Box::new(lpt::LPT {}),
                    Box::new(lptp::Lptp {}),
                ];

                let (mut lower_bound, mut upper_bound) = (0, None);
                for bound in bounds {
                    (lower_bound, upper_bound) = bound.bound(&instance, lower_bound, upper_bound);
                }
                let upper_bound = upper_bound.unwrap();
                let pi = PartialSolution::new(instance);
                for makespan in lower_bound..upper_bound.makespan + 1 {
                    file_num += 1;
                    encoder.basic_encode(&pi, makespan);
                    let e = encoder.output();
                    input_output::to_dimacs::print_to_dimacs(
                        &(out_dirname.to_owned() + &file_num.to_string() + ".txt"),
                        e,
                        encoder.get_num_vars(),
                    );
                }
            }
        }
    }

    #[test]
    #[ignore]
    pub fn test_basic() {
        let mut encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        test_encoder(
            &mut encoder,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/basic/",
        )
    }

    //#[test]
    //#[ignore]
    //pub fn test_furlite() {
    //    let mut a: Box<dyn Encoder> = Box::new(FillUpLite::new());
    //    test_encoder(
    //        &mut a,
    //        "./bench/class_instances/",
    //        "./bench/class_instance_encodings/furlite/",
    //    )
    //}

    #[test]
    #[ignore]
    pub fn test_basic_with_precedense() {
        let basic: Box<dyn OneHotEncoder> = Box::new(BasicEncoder::new());
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(basic));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/basic_precedence/",
        )
    }

    #[test]
    #[ignore]
    pub fn test_pysat() {
        let mut a: Box<dyn Encoder> = Box::new(PbPysatEncoder::new());
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/binmerge/",
        )
    }

    #[test]
    #[ignore]
    pub fn test_pysat_with_precedence() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbPysatEncoder::new())));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/binmerge_precedence_one/",
        )
    }

    #[test]
    #[ignore]
    pub fn test_bdd_native() {
        let mut a: Box<dyn Encoder> = Box::new(PbNativeEncoder::new());
        test_encoder(
            &mut a,
            //"./bench/single_instance/",
            //"./bench/single_instance/"
            "./bench/class_instances/",
            "./bench/class_instance_encodings/bdd/",
        )
    }

    #[test]
    //#[ignore]
    pub fn test_bdd_native_with_precedence() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbNativeEncoder::new())));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/bdd_precedence_old/",
        )
    }

    #[test]
    #[ignore]
    pub fn test_random() {
        let a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbPysatEncoder::new())));
        let mut a: Box<dyn Encoder> = Box::new(RandomEncoder::new(a, 0.5));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/random/",
        )
    }

    //#[test]
    //#[ignore]
    //pub fn test_bdd_native_f_e() {
    //    let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbNativeEncoderFE::new())));
    //    test_encoder(
    //        &mut a,
    //        "./bench/class_instances/",
    //        "./bench/class_instance_encodings/bdd_fe/",
    //    )
    //}

    fn test_unsat_ness(encoder: &mut Box<dyn Encoder>, in_dirname: &str, out_dirname: &str) {
        //
        let paths = fs::read_dir(in_dirname).unwrap();
        let mut file_num = 0;
        let mut instance_num = 0;
        for path in paths {
            let path = path.as_ref().unwrap().path().display().to_string();
            if path.ends_with(".txt") && !path.contains("n100") && !path.contains("n50") {
                
                let instance = input_output::from_file::read_from_file(&path);
                let bounds: Vec<Box<dyn Bound>> = vec![
                    Box::new(pigeon_hole::PigeonHole {}),
                    Box::new(max_job_size::MaxJobSize {}),
                    Box::new(middle::MiddleJobs {}),
                    Box::new(lpt::LPT {}),
                    Box::new(lptp::Lptp {}),
                    Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
                    Box::new(lptpp::Lptpp {}),
                ];

                let (mut lower_bound, mut upper_bound) = (0, None);
                for bound in bounds {
                    (lower_bound, upper_bound) = bound.bound(&instance, lower_bound, upper_bound);
                    println!("lower: {} upper {}", lower_bound, if upper_bound.is_some() {upper_bound.as_ref().unwrap().makespan} else {0});
                }

                let pi = PartialSolution::new(instance);
                let max_job_size = pi.instance.job_sizes[0];
                for makespan in (lower_bound - 10)..lower_bound + 1 {
                    if file_num > 500 {
                        return;
                    }
                    if makespan < max_job_size {
                        continue;
                    }
                    file_num += 1;
                    encoder.basic_encode(&pi, makespan);
                    let e = encoder.output();
                    input_output::to_dimacs::print_to_dimacs(
                        &(out_dirname.to_owned()
                            + &instance_num.to_string()
                            + "_"
                            + &(lower_bound - makespan).to_string()
                            + ".txt"),
                        e,
                        encoder.get_num_vars(),
                    );
                }
                instance_num += 1;
            }
        }
    }

    #[test]
    #[ignore]
    pub fn test_bdd_unsatness() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbNativeEncoder::new())));
        test_unsat_ness(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/unsatness_bdd/",
        )
    }

    #[test]
    #[ignore]
    pub fn test_binmerge_unsatness() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbPysatEncoder::new())));
        test_unsat_ness(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/unsatness_binmerge/",
        )
    }
}
