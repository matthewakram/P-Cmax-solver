
#[cfg(test)]
mod tests {
    use crate::{encoding::{binary_arithmetic, encoder::{VarNameGenerator, Clauses, Clause}, cardinality_networks::{basic_sort, basic_merge, half_merge, half_sort}}, input_output::to_dimacs, common::timeout::Timeout, solvers::{sat_solver::kissat::Kissat, solver::SatSolver}};

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
        //assert_eq!(binary_arithmetic::at_most_k_encoding(&number, 5), vec![Clause{vars : vec![-2, -3]}, Clause{vars : vec![-4]}]);
    }

    #[test]
    pub fn binary_arithmetic_sum_test() {
        let mut name_generator = VarNameGenerator::new();
        let number1: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(8, &mut name_generator);
        let number2: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(7, &mut name_generator);
        //let number3: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(15, &mut name_generator);
        //let number4: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(15, &mut name_generator);
        assert_eq!(number1.bit_length, 4);
        assert_eq!(number2.bit_length, 3);

        let mut clauses: Clauses = Clauses::new();

        let (sum, sum_assertion) = binary_arithmetic::bounded_sum_encoding(&number1, &number2, 5, &mut name_generator);
        clauses.add_many_clauses(&mut sum_assertion.clone());
        //let (sum, sum_assertion) = binary_arithmetic::bounded_sum_encoding(&sum, &number2, 4, &mut name_generator);
        //clauses.append(&mut sum_assertion.clone());
        //let (sum, sum_assertion) = binary_arithmetic::bounded_sum_encoding(&sum, &number3, 4, &mut name_generator);
        //clauses.append(&mut sum_assertion.clone());
        //let (sum, sum_assertion) = binary_arithmetic::bounded_sum_encoding(&sum, &number4, 4, &mut name_generator);
        //clauses.append(&mut sum_assertion.clone());

        clauses.add_many_clauses(&mut binary_arithmetic::_equals_constant_encoding(&number1, 11));
        clauses.add_many_clauses(&mut binary_arithmetic::_equals_constant_encoding(&number2, 7));

        //let mut leq_clauses = binary_arithmetic::at_most_k_encoding(&sum, 11);
        //clauses.append(&mut leq_clauses);
        //let sum_equals_value = binary_arithmetic::not_equals_constant_encoding(&sum, 18);
        //clauses.push(sum_equals_value);
        let mut sum_equals_value = binary_arithmetic::_equals_constant_encoding(&sum, 2);
        clauses.add_many_clauses(&mut sum_equals_value);

        to_dimacs::_print_to_dimacs("./test", clauses, 100, &Timeout::new(1000.0)
);
    }

    #[test]
    pub fn binary_arithmeticn_equals_i_implies_m_in_j_encoding_test() {
        let mut name_generator = VarNameGenerator::new();
        let number1: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(50, &mut name_generator);
        let number2: binary_arithmetic::BinaryNumber = binary_arithmetic::BinaryNumber::new(31, &mut name_generator);
        assert_eq!(number1.bit_length, 6);
        assert_eq!(number2.bit_length, 5);

        let mut clauses = binary_arithmetic::n_equals_i_implies_m_in_j_encoding(&number1, 6, &number2, &vec![10, 20, 30]);

        clauses.add_many_clauses(&mut binary_arithmetic::_equals_constant_encoding(&number1, 6));
        clauses.add_clause(binary_arithmetic::not_equals_constant_encoding(&number2, 10));
        clauses.add_clause(binary_arithmetic::not_equals_constant_encoding(&number2, 20));
        clauses.add_clause(binary_arithmetic::not_equals_constant_encoding(&number2, 30));


        //to_dimacs::print_to_dimacs("./test", clauses, 100);
    }

    #[test]
    pub fn simp_sorter_test(){
        let mut name_generator = VarNameGenerator::new();
        let vars = vec![name_generator.next(), name_generator.next(), name_generator.next() , name_generator.next(), name_generator.next()];
        let (mut clauses, sorted) = basic_sort(&vars, 5, &mut name_generator);
        clauses.add_clause(Clause{ vars: vec![(vars[0] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars[1] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars[2] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars[3] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars[4] as i32)]});

        let mut solver = Kissat::new();
        let res = solver.solve(clauses, name_generator.peek(), &Timeout::new(10.0));
        assert!(res.is_sat());
        let res = res.unwrap().unwrap();
        let mut result = vec![0;20];
        for i in 0..20 {
            if res.contains(&(i as i32)) {
                result[i] = 1;
            }
        }
        println!("{:?}", result);
    }

    #[test]
    pub fn merge_test(){
        let mut name_generator = VarNameGenerator::new();
        let vars1 = vec![name_generator.next(), name_generator.next(), name_generator.next() , name_generator.next(), name_generator.next()];
        let vars2 = vec![name_generator.next(), name_generator.next(), name_generator.next() , name_generator.next(), name_generator.next()];
        
        let (mut clauses, merged) = basic_merge(&vars1, &vars2, 5, &mut name_generator);

        clauses.add_clause(Clause{ vars: vec![-(vars1[0] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars1[1] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars1[2] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars1[3] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars1[4] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars2[0] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars2[1] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars2[2] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars2[3] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars2[4] as i32)]});

        let mut solver = Kissat::new();
        let res = solver.solve(clauses, name_generator.peek(), &Timeout::new(10.0));
        assert!(res.is_sat());
        let res = res.unwrap().unwrap();
        let mut result = vec![0;merged.len()];
        for i in 0..merged.len() {
            if res.contains(&(merged[i] as i32)) {
                result[i] = 1;
            }
        }
        println!("{:?}", result);
    }

    #[test]
    pub fn half_merge_test(){
        let mut name_generator = VarNameGenerator::new();
        let vars1 = vec![name_generator.next(), name_generator.next(), name_generator.next(), name_generator.next()];
        let vars2 = vec![name_generator.next(), name_generator.next()];
        
        let (mut clauses, merged) = half_merge(&vars1, &vars2, 3, &mut name_generator);

        clauses.add_clause(Clause{ vars: vec![(vars1[0] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars1[1] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars1[2] as i32)]});
        //clauses.add_clause(Clause{ vars: vec![-(vars1[3] as i32)]});
        //clauses.add_clause(Clause{ vars: vec![-(vars1[4] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars2[0] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(vars2[1] as i32)]});
        //clauses.add_clause(Clause{ vars: vec![-(vars2[2] as i32)]});
        //clauses.add_clause(Clause{ vars: vec![-(vars2[3] as i32)]});
        //clauses.add_clause(Clause{ vars: vec![-(vars2[4] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(merged[0] as i32)]});
        //clauses.add_clause(Clause{ vars: vec![(merged[1] as i32)]});
        clauses.add_clause(Clause{ vars: vec![-(merged[2] as i32)]});
        //clauses.add_clause(Clause{ vars: vec![-(merged[3] as i32)]});
        //clauses.add_clause(Clause{ vars: vec![-(merged[4] as i32)]});

        let mut solver = Kissat::new();
        let res = solver.solve(clauses.clone(), name_generator.peek(), &Timeout::new(10.0));
        assert!(res.is_sat());
        let res = res.unwrap().unwrap();
        let mut result = vec![0;merged.len()];
        for i in 0..merged.len() {
            if res.contains(&(merged[i] as i32)) {
                result[i] = 1;
            }
        }
        println!("{:?}", result);
    }

    #[test]
    pub fn half_sorter_test(){
        let mut name_generator = VarNameGenerator::new();
        let vars = vec![name_generator.next(), name_generator.next(), name_generator.next() , name_generator.next(), name_generator.next()];
        let (mut clauses, sorted) = half_sort(&vars, 4, &mut name_generator);
        clauses.add_clause(Clause{ vars: vec![(vars[0] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars[1] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars[2] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars[3] as i32)]});
        clauses.add_clause(Clause{ vars: vec![(vars[4] as i32)]});

        let mut solver = Kissat::new();
        let res = solver.solve(clauses, name_generator.peek(), &Timeout::new(10.0));
        assert!(res.is_sat());
        let res = res.unwrap().unwrap();
        let mut result = vec![0;20];
        for i in 0..20 {
            if res.contains(&(i as i32)) {
                result[i] = 1;
            }
        }
        println!("{:?}", result);
    }

}
