
#[cfg(test)]
mod tests {
    use core::num;

    use crate::{encoding::{binary_arithmetic, encoder::{VarNameGenerator, Clause}}, input_output::to_dimacs};

    #[test]
    pub fn binary_arithmetic_bit_length_test() {
        assert!(binary_arithmetic::number_bitlength(1) == 1);
        assert!(binary_arithmetic::number_bitlength(2) == 2);
        assert!(binary_arithmetic::number_bitlength(3) == 2);
        assert!(binary_arithmetic::number_bitlength(5) == 3);
        assert!(binary_arithmetic::number_bitlength(8) == 4);
        assert!(binary_arithmetic::number_bitlength(15) == 4);
        assert!(binary_arithmetic::number_bitlength(16) == 5);
        assert!(binary_arithmetic::number_bitlength(17) == 5);
    }

    #[test]
    pub fn binary_arithmetic_repr_test() {
        assert_eq!(binary_arithmetic::to_binary(4), vec![false, false, true]);
        assert_eq!(binary_arithmetic::to_binary(5), vec![true, false, true]);
        assert_eq!(binary_arithmetic::to_binary(15), vec![true, true, true, true]);
        assert_eq!(binary_arithmetic::to_binary(16), vec![false, false, false, false, true]);
        assert_eq!(binary_arithmetic::to_binary(17), vec![true, false, false, false, true]);
        assert_eq!(binary_arithmetic::to_binary(21), vec![true, false, true, false, true]);
    }

    #[test]
    pub fn binary_arithmetic_leq_test() {
        let mut name_generator = VarNameGenerator::new();
        let number = binary_arithmetic::BinaryNumber::new(15, &mut name_generator);
        assert_eq!(number.bit_length, 4);
        assert_eq!(binary_arithmetic::at_most_k_encoding(&number, 5), vec![Clause{vars : vec![-2, -3]}, Clause{vars : vec![-4]}]);
    }

    #[test]
    pub fn binary_arithmetic_sum_test() {
        let mut name_generator = VarNameGenerator::new();
        let number1: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(50, &mut name_generator);
        let number2: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(31, &mut name_generator);
        assert_eq!(number1.bit_length, 6);
        assert_eq!(number2.bit_length, 5);
        let (sum, sum_assertion) = binary_arithmetic::bounded_sum_encoding(&number1, &number2, 7, &mut name_generator);

        let mut should_be_solvable_clauses: Vec<Clause> = vec![];

        should_be_solvable_clauses.append(&mut binary_arithmetic::equals_constant_encoding(&number1, 49));
        should_be_solvable_clauses.append(&mut binary_arithmetic::equals_constant_encoding(&number2, 25));
        should_be_solvable_clauses.append(&mut sum_assertion.clone());

        let mut not_solvable = should_be_solvable_clauses.clone();
        let sum_equals_value = binary_arithmetic::equals_constant_encoding(&sum, 74);
        should_be_solvable_clauses.append(&mut sum_equals_value.clone());

        not_solvable.push(binary_arithmetic::not_equals_constant_encoding(&sum, 74));

        //to_dimacs::print_to_dimacs("./should_be_solvable", should_be_solvable_clauses, 100);
        //to_dimacs::print_to_dimacs("./should_not_be_solvable", not_solvable, 100);
    }

    #[test]
    pub fn binary_arithmeticn_equals_i_implies_m_in_j_encoding_test() {
        let mut name_generator = VarNameGenerator::new();
        let number1: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(50, &mut name_generator);
        let number2: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(31, &mut name_generator);
        assert_eq!(number1.bit_length, 6);
        assert_eq!(number2.bit_length, 5);

        let mut clauses = binary_arithmetic::n_equals_i_implies_m_in_j_encoding(&number1, 6, &number2, &vec![10, 20, 30]);

        clauses.append(&mut binary_arithmetic::equals_constant_encoding(&number1, 6));
        clauses.push(binary_arithmetic::not_equals_constant_encoding(&number2, 10));
        clauses.push(binary_arithmetic::not_equals_constant_encoding(&number2, 20));
        clauses.push(binary_arithmetic::not_equals_constant_encoding(&number2, 30));


        to_dimacs::print_to_dimacs("./test", clauses, 100);
    }
}
