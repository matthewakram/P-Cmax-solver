#[cfg(test)]
mod tests {
    use bitvec::vec;
    use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

    use crate::{
        common::timeout::Timeout,
        encoding::{
            binary_arithmetic::{self, BinaryNumber},
            cardinality_networks::{self, basic_merge, basic_sort, half_merge, half_sort},
            encoder::{Clause, Clauses, VarNameGenerator},
        },
        solvers::{sat_solver::kissat::Kissat, solver::SatSolver},
    };

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
        assert_eq!(
            binary_arithmetic::to_binary(15),
            vec![true, true, true, true]
        );
        assert_eq!(
            binary_arithmetic::to_binary(16),
            vec![false, false, false, false, true]
        );
        assert_eq!(
            binary_arithmetic::to_binary(17),
            vec![true, false, false, false, true]
        );
        assert_eq!(
            binary_arithmetic::to_binary(21),
            vec![true, false, true, false, true]
        );
    }

    #[test]
    pub fn binary_arithmetic_leq_test() {
        let mut name_generator = VarNameGenerator::new();
        let number = binary_arithmetic::BinaryNumber::new(15, &mut name_generator);
        assert_eq!(number.bit_length, 4);
        let mut expected_clauses = Clauses::new();
        expected_clauses.add_clause(Clause { vars: vec![-2, -3] });
        expected_clauses.add_clause(Clause { vars: vec![-4] });
        assert_eq!(
            binary_arithmetic::at_most_k_encoding(&number, 5),
            expected_clauses
        );
    }

    #[test]
    pub fn binary_arithmetic_sum_test() {
        let mut solver = Kissat::new();

        for i in 1..50 {
            for j in 1..50 {
                let mut name_generator = VarNameGenerator::new();
                let number1 = BinaryNumber::new(i, &mut name_generator);
                let number2 = BinaryNumber::new(j, &mut name_generator);

                let mut clauses = binary_arithmetic::_equals_constant_encoding(&number1, i);
                clauses.add_many_clauses(&mut binary_arithmetic::_equals_constant_encoding(
                    &number2, j,
                ));
                let (sum, mut sum_clauses) = binary_arithmetic::bounded_sum_encoding(
                    &number1,
                    &number2,
                    10,
                    &mut name_generator,
                );
                clauses.add_many_clauses(&mut sum_clauses);

                let mut sat_clauses = clauses.clone();
                let mut unsat_clauses = clauses;

                sat_clauses
                    .add_many_clauses(&mut binary_arithmetic::at_most_k_encoding(&sum, i + j));
                unsat_clauses
                    .add_many_clauses(&mut binary_arithmetic::at_most_k_encoding(&sum, i + j - 1));

                let sat = solver.solve(sat_clauses, 100, &Timeout::new(5.0));
                let unsat = solver.solve(unsat_clauses, 100, &Timeout::new(5.0));

                println!("{} + {}", i, j);
                assert!(sat.is_sat());
                assert!(unsat.is_unsat());
            }
        }
    }

    #[test]
    pub fn binary_arithmeticn_equals_i_implies_m_in_j_encoding_test() {
        let mut name_generator = VarNameGenerator::new();
        let number1: binary_arithmetic::BinaryNumber =
            binary_arithmetic::BinaryNumber::new(50, &mut name_generator);
        let number2: binary_arithmetic::BinaryNumber =
            binary_arithmetic::BinaryNumber::new(31, &mut name_generator);
        assert_eq!(number1.bit_length, 6);
        assert_eq!(number2.bit_length, 5);

        let mut clauses = binary_arithmetic::n_equals_i_implies_m_in_j_encoding(
            &number1,
            6,
            &number2,
            &vec![10, 20, 30],
        );

        let sat_clauses1 = clauses.clone();

        clauses.add_many_clauses(&mut binary_arithmetic::_equals_constant_encoding(
            &number1, 6,
        ));

        let sat_clauses2 = clauses.clone();
        clauses.add_clause(binary_arithmetic::not_equals_constant_encoding(
            &number2, 10,
        ));
        let sat_clauses3 = clauses.clone();

        clauses.add_clause(binary_arithmetic::not_equals_constant_encoding(
            &number2, 20,
        ));

        let sat_clauses4 = clauses.clone();
        clauses.add_clause(binary_arithmetic::not_equals_constant_encoding(
            &number2, 30,
        ));

        let unsat_clauses = clauses.clone();

        let mut solver = Kissat::new();
        let sat1 = solver.solve(sat_clauses1, 100, &Timeout::new(5.0));
        let sat2 = solver.solve(sat_clauses2, 100, &Timeout::new(5.0));
        let sat3 = solver.solve(sat_clauses3, 100, &Timeout::new(5.0));
        let sat4 = solver.solve(sat_clauses4, 100, &Timeout::new(5.0));
        let unsat = solver.solve(unsat_clauses, 100, &Timeout::new(5.0));
        assert!(sat1.is_sat());
        assert!(sat2.is_sat());
        assert!(sat3.is_sat());
        assert!(sat4.is_sat());
        assert!(unsat.is_unsat());
    }

    #[test]
    pub fn simp_sorter_test() {
        let mut rng = ThreadRng::default();
        let mut solver = Kissat::new();
        for i in 1..10 {
            for j in 1..i + 1 {
                let mut clauses = Clauses::new();
                let mut name_generator = VarNameGenerator::new();
                let mut vars: Vec<usize> =
                    (0..i).into_iter().map(|_| name_generator.next()).collect();
                for k in 0..j {
                    clauses.add_clause(Clause {
                        vars: vec![vars[k] as i32],
                    });
                }
                vars.shuffle(&mut rng);

                let mut sat_clauses = clauses.clone();
                let mut unsat_clauses = clauses;

                let (mut sorted_clauses, sorted_vars) =
                    cardinality_networks::basic_sort(&vars, j, &mut name_generator);
                sat_clauses.add_many_clauses(&mut sorted_clauses);

                let (mut sorted_clauses, _) =
                    cardinality_networks::basic_sort(&vars, j - 1, &mut name_generator);
                unsat_clauses.add_many_clauses(&mut sorted_clauses);

                let sat = solver.solve(sat_clauses, name_generator.peek(), &Timeout::new(1.0));
                let unsat = solver.solve(unsat_clauses, name_generator.peek(), &Timeout::new(1.0));
                assert!(sat.is_sat());
                assert!(unsat.is_unsat());

                let assignment = sat.unwrap().unwrap();
                for k in 0..j {
                    assert!(assignment.contains(&(sorted_vars[k] as i32)));
                }
            }
        }
    }

    #[test]
    pub fn merge_test() {
        let mut rng = ThreadRng::default();
        let mut solver = Kissat::new();
        for i in 0..20 {
            for j in 0..20 {
                let mut clauses = Clauses::new();
                let mut var_name_generator = VarNameGenerator::new();
                let vars1: Vec<usize> = (0..i)
                    .into_iter()
                    .map(|_| var_name_generator.next())
                    .collect();
                let vars2: Vec<usize> = (0..j)
                    .into_iter()
                    .map(|_| var_name_generator.next())
                    .collect();

                let num_true_1 = if i == 0 { 0 } else { rng.gen_range(0..i) };
                let num_true_2 = if j == 0 { 0 } else { rng.gen_range(0..j) };

                for k in 0..num_true_1 {
                    clauses.add_clause(Clause {
                        vars: vec![vars1[k] as i32],
                    });
                }
                for k in 0..num_true_2 {
                    clauses.add_clause(Clause {
                        vars: vec![vars2[k] as i32],
                    });
                }

                let (mut merge_clauses, merged_vars) =
                    cardinality_networks::basic_merge(&vars1, &vars2, 10, &mut var_name_generator);
                clauses.add_many_clauses(&mut merge_clauses);
                let sol = solver.solve(clauses, var_name_generator.peek(), &Timeout::new(1.0));
                if num_true_1 + num_true_2 > 10 {
                    assert!(sol.is_unsat());
                } else {
                    assert!(sol.is_sat());
                    let assignment = sol.unwrap().unwrap();
                    for i in 0..num_true_1 + num_true_2 {
                        assert!(assignment.contains(&(merged_vars[i] as i32)));
                    }
                }
            }
        }
    }

    #[test]
    pub fn half_merge_test() {
        let mut rng = ThreadRng::default();
        let mut solver = Kissat::new();
        for i in 0..20 {
            for j in 0..20 {
                let mut clauses = Clauses::new();
                let mut var_name_generator = VarNameGenerator::new();
                let vars1: Vec<usize> = (0..i)
                    .into_iter()
                    .map(|_| var_name_generator.next())
                    .collect();
                let vars2: Vec<usize> = (0..j)
                    .into_iter()
                    .map(|_| var_name_generator.next())
                    .collect();

                let num_true_1 = if i == 0 { 0 } else { rng.gen_range(0..i) };
                let num_true_2 = if j == 0 { 0 } else { rng.gen_range(0..j) };

                for k in 0..num_true_1 {
                    clauses.add_clause(Clause {
                        vars: vec![vars1[k] as i32],
                    });
                }
                for k in 0..num_true_2 {
                    clauses.add_clause(Clause {
                        vars: vec![vars2[k] as i32],
                    });
                }

                let (mut merge_clauses, merged_vars) =
                    cardinality_networks::half_merge(&vars1, &vars2, 10, &mut var_name_generator);
                clauses.add_many_clauses(&mut merge_clauses);
                let sol = solver.solve(clauses.clone(), var_name_generator.peek(), &Timeout::new(1.0));
                if num_true_1 + num_true_2 > 10 {
                    assert!(sol.is_unsat());
                } else {
                    assert!(sol.is_sat());
                    let assignment = sol.unwrap().unwrap();
                    for i in 0..num_true_1 + num_true_2 {
                        assert!(assignment.contains(&(merged_vars[i] as i32)));
                    }
                }
            }
        }
    }

    #[test]
    pub fn half_sorter_test() {
        let mut rng = ThreadRng::default();
        let mut solver = Kissat::new();
        for i in 1..10 {
            for j in 1..i + 1 {
                let mut clauses = Clauses::new();
                let mut name_generator = VarNameGenerator::new();
                let mut vars: Vec<usize> =
                    (0..i).into_iter().map(|_| name_generator.next()).collect();
                for k in 0..j {
                    clauses.add_clause(Clause {
                        vars: vec![vars[k] as i32],
                    });
                }
                vars.shuffle(&mut rng);

                let mut sat_clauses = clauses.clone();
                let mut unsat_clauses = clauses;

                let (mut sorted_clauses, sorted_vars) =
                    cardinality_networks::half_sort(&vars, j, &mut name_generator);
                sat_clauses.add_many_clauses(&mut sorted_clauses);

                let (mut sorted_clauses, _) =
                    cardinality_networks::basic_sort(&vars, j - 1, &mut name_generator);
                unsat_clauses.add_many_clauses(&mut sorted_clauses);

                let sat = solver.solve(sat_clauses, name_generator.peek(), &Timeout::new(1.0));
                let unsat = solver.solve(unsat_clauses, name_generator.peek(), &Timeout::new(1.0));
                assert!(sat.is_sat());
                assert!(unsat.is_unsat());

                let assignment = sat.unwrap().unwrap();
                for k in 0..j {
                    assert!(assignment.contains(&(sorted_vars[k] as i32)));
                }
            }
        }
    }
}
