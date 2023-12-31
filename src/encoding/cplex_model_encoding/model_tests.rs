#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    #[test]
    pub fn pinar_seyda_test(){
        use crate::{problem_instance::{problem_instance::ProblemInstance, partial_solution::PartialSolution}, encoding::{cplex_model_encoding::pinar_seyda::PinarSeyda, cplex_model_encoder::CPLEXModelEncoder}, common::timeout::Timeout};

        let instance = ProblemInstance::new(3, 10, vec![1, 2, 3, 4, 5, 6, 7, 8 ,9 ,10]);
        let mut encoder = PinarSeyda::new();
        let pi = PartialSolution::new(instance);
        encoder.encode(&pi, 19, 25, &Timeout::new(3.0));
        let out = encoder.get_encoding();
        let mut file = File::create("./test.data").unwrap();
        file.write(out.as_bytes()).unwrap();
    }
}