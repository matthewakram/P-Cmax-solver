
#[cfg(test)]
mod tests {
    use crate::encoding::{binary_arithmetic, encoder::{VarNameGenerator, Clause}};

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
        let number1: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(7, &mut name_generator);
        let number2 = binary_arithmetic::BinaryNumber::new(7, &mut name_generator);
        assert_eq!(number1.bit_length, 3);
        let a = binary_arithmetic::bounded_sum_encoding(&number1, &number2, 3, &mut name_generator);

        println!("{:?}", a);
    }
}
