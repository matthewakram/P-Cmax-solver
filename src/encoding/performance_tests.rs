#[cfg(test)]
mod tests {
    use crate::{
        bounds::{
            lower_bounds::{
                self,
                lower_bound::{self, LowerBound},
            },
            upper_bounds::{self, upper_bound::InitialUpperBound},
        },
        encoding::{
            encoder::{Clause, Encoder, VarNameGenerator, self}, basic_encoder::BasicEncoder, furlite_with_precedence::FurliteWithPrecedence, fill_up_lite::FillUpLite,
        },
        input_output::{self, to_dimacs}, problem_instance::partial_solution::PartialSolution, precedence_relations::{self, precedence_relation_generator::PrecedenceRelationGenerator},
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
                let initial_lower_bound = lower_bounds::pigeon_hole::PigeonHole {};
                let initial_lower_bound = initial_lower_bound.get_lower_bound(&instance);
                let initial_lower_bound2 = lower_bounds::max_job_size::MaxJobSize {};
                let initial_lower_bound2 = initial_lower_bound2.get_lower_bound(&instance);
                let initial_lower_bound = initial_lower_bound.max(initial_lower_bound2);

                let initial_upper_bound = upper_bounds::lptpp::Lptpp { lower_bound: initial_lower_bound };
                let initial_upper_bound = initial_upper_bound.get_upper_bound(&instance).makespan;
                
                let pi = PartialSolution::new(instance);
                for makespan in initial_lower_bound..initial_upper_bound + 1 {
                    file_num += 1;
                    encoder.basic_encode(&pi, makespan);
                    let e = encoder.output();
                    input_output::to_dimacs::print_to_dimacs(&(out_dirname.to_owned() + &file_num.to_string() + ".txt"), e, encoder.get_num_vars());
                }
            }
        }
    }

    #[test]
    #[ignore]
    pub fn test_basic(){
        let mut encoder: Box<dyn Encoder> = Box::new(BasicEncoder::new());
        test_encoder(&mut encoder, "./bench/class_instances/", "./bench/class_instance_results/")
    }

    #[test]
    //#[ignore]
    pub fn test_precedense(){
        let mut a: Box<dyn Encoder> = Box::new(FurliteWithPrecedence::new());
        test_encoder(&mut a,"./bench/class_instances/", "./bench/class_instance_results_prec/")
    }

    #[test]
    #[ignore]
    pub fn test_furlite(){
        let mut a: Box<dyn Encoder> = Box::new(FillUpLite::new());
        test_encoder(&mut a,"./bench/class_instances/", "./bench/class_instance_results_furlite/")
    }

}
