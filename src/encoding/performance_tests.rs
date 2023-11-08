#[cfg(test)]
mod tests {
    use crate::{
        bounds::{
            lower_bounds::{
                pigeon_hole,
                sss_bound_tightening::{self}, max_job_size, middle,
            },
            upper_bounds::{lpt, lptp, lptpp}, bound::Bound,
        },
        encoding::{
            basic_encoder::BasicEncoder,
            basic_with_precedence::Precedence,
            encoder::{Encoder, OneHotEncoder},
            pb_bdd_native::PbNativeEncoder,
            pb_bdd_pysat::PbPysatEncoder,
            random_encoder::RandomEncoder, pb_bdd_inter::PbInter,
        },
        input_output::{self},
        problem_instance::partial_solution::PartialSolution, problem_simplification::{final_simp_rule, simplification_rule::SimpRule},
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
                    Box::new(lptpp::Lptpp {})
                ];

                let (mut lower_bound, mut upper_bound) = (0, None);
                for bound in bounds {
                    (lower_bound, upper_bound) = bound.bound(&instance, lower_bound, upper_bound,  10000.0);
                }
                let upper_bound = upper_bound.unwrap();
                let pi = PartialSolution::new(instance);
                
                for makespan in lower_bound..upper_bound.makespan + 1 {
                    let mut finalize: final_simp_rule::FinalizeRule = final_simp_rule::FinalizeRule{};
                    let pi = finalize.simplify(&pi, makespan);
                    if pi.is_none() {
                        continue;
                    }
                    let pi: PartialSolution = pi.unwrap();
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
    //#[ignore]
    pub fn test_basic() {
        let mut encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        test_encoder(
            &mut encoder,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/basic/",
        )
    }


    #[test]
    #[ignore]
    pub fn test_basic_with_precedense() {
        let basic: Box<dyn OneHotEncoder> = Box::new(BasicEncoder::new());
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(basic, 2));
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
    //#[ignore]
    pub fn test_pysat_with_precedence() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbPysatEncoder::new()), 2));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/binmerge_precedence/",
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
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbNativeEncoder::new()), 2));
        test_encoder(
            &mut a,
            //"./bench/single_instance/",
            //"./bench/single_instance/" 
            "./bench/class_instances/",
            "./bench/class_instance_encodings/bdd_precedence/",
        )
    }

    #[test]
    #[ignore]
    pub fn test_inter_with_precedence_unopt() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::_new_unopt()), 2));
        test_encoder(
            &mut a,
            //"./bench/single_instance/",
            //"./bench/single_instance/"
            "./bench/class_instances/",
            "./bench/class_instance_encodings/bdd_inter_precedence_unopt/",
        )
    }

    #[test]
    //#[ignore]
    pub fn test_inter_with_precedence() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::new()), 2));
        test_encoder(
            &mut a,
            //"./bench/single_instance/",
            //"./bench/single_instance/"
            "./bench/class_instances/",
            "./bench/class_instance_encodings/bdd_inter_precedence/",
        )
    }

    #[test]
    //#[ignore]
    pub fn test_inter_precedence_1() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbInter::new()), 1));
        test_encoder(
            &mut a,
            //"./bench/single_instance/",
            //"./bench/single_instance/"
            "./bench/class_instances/",
            "./bench/class_instance_encodings/bdd_inter_prec_1/",
        )
    }

    #[test]
    #[ignore]
    pub fn test_inter() {
        let mut a: Box<dyn Encoder> = Box::new(PbInter::new());
        test_encoder(
            &mut a,
            //"./bench/single_instance/",
            //"./bench/single_instance/"
            "./bench/class_instances/",
            "./bench/class_instance_encodings/bdd_inter/",
        )
    }

    #[test]
    #[ignore]
    pub fn test_random() {
        let a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbPysatEncoder::new()), 2));
        let mut a: Box<dyn Encoder> = Box::new(RandomEncoder::new(a, 0.5));
        test_encoder(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/random/",
        )
    }


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
                    (lower_bound, upper_bound) = bound.bound(&instance, lower_bound, upper_bound, 10000.0);
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
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbNativeEncoder::new()), 2));
        test_unsat_ness(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/unsatness_bdd/",
        )
    }

    #[test]
    #[ignore]
    pub fn test_binmerge_unsatness() {
        let mut a: Box<dyn Encoder> = Box::new(Precedence::new(Box::new(PbPysatEncoder::new()), 2));
        test_unsat_ness(
            &mut a,
            "./bench/class_instances/",
            "./bench/class_instance_encodings/unsatness_binmerge/",
        )
    }
}
