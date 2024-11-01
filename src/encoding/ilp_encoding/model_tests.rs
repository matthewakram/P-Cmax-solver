#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use crate::encoding::ilp_encoder::ILPEncoder;
    use crate::encoding::ilp_encoding::mehdi_nizar_original::MehdiNizarOriginalEncoder;
    use crate::encoding::ilp_encoding::mehdi_nizar_prec::MehdiNizarOrderEncoder;
    use crate::{
        common::timeout::Timeout,
        problem_instance::{partial_solution::PartialSolution, problem_instance::ProblemInstance},
    };

    #[test]
    pub fn mehdi_nizar_test() {
        let instance = ProblemInstance::new(3, 10, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut encoder = MehdiNizarOrderEncoder::new();
        let pi = PartialSolution::new(instance);
        encoder.encode(&pi, 19, 25, &Timeout::new(3.0));
        let out = encoder.get_encoding();
        let mut file = File::create("./test.lp").unwrap();
        println!("{}", out);
        file.write(out.as_bytes()).unwrap();
    }

    #[test]
    pub fn mehdi_nizar_original_test() {
        let instance = ProblemInstance::new(3, 10, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut encoder = MehdiNizarOriginalEncoder::new();
        let pi = PartialSolution::new(instance);
        encoder.encode(&pi, 19, 25, &Timeout::new(3.0));
        let out = encoder.get_encoding();
        let mut file = File::create("./test.lp").unwrap();
        println!("{}", out);
        file.write(out.as_bytes()).unwrap();
    }
}
