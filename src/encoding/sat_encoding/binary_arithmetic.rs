use crate::encoding::encoder::{VarNameGenerator, Clauses, Clause};


#[derive(Debug, Clone)]
pub struct BinaryNumber {
    /// the sat variable representation of a number
    pub vars: Vec<usize>,
    /// max is inclusive. The represented number can be == max
    pub max: usize,
    /// the number of bits used to represent this number
    pub bit_length: usize,
}

impl BinaryNumber {
    pub fn new(max: usize, name_generator: &mut VarNameGenerator) -> BinaryNumber {
        let bit_length = number_bitlength(max);
        let vars: Vec<usize> = (0..bit_length).map(|_| name_generator.next()).collect();

        return BinaryNumber {
            vars,
            max,
            bit_length,
        };
    }

    pub fn new_from_vec(vars: Vec<usize>, max: usize) -> BinaryNumber {
        let bit_length = vars.len();
        return BinaryNumber {
            vars,
            max,
            bit_length,
        };
    }

    pub fn _from_assignment(&self, assignment: &Vec<i32>) -> usize {
        let mut total: usize = 0;
        for bit in 0..self.bit_length {
            if assignment.contains(&(self.vars[bit] as i32)) {
                total += 1 << bit;
            }
        }
        return total;
    }
}

pub fn number_bitlength(k: usize) -> usize {
    if k == 0 {
        return 1;
    }
    let mut bitlength: usize = 1;

    while (1 << bitlength) <= k {
        bitlength += 1;
    }
    assert!(k < (1 << bitlength));
    assert!(k > (1 << (bitlength - 1)) - 1);
    return bitlength;
}

pub fn to_binary(k: usize) -> Vec<bool> {
    let bit_length = number_bitlength(k);

    let mut result: Vec<bool> = vec![];
    for i in 0..bit_length {
        result.push((k & (1 << i)) > 0);
    }
    return result;
}

pub fn at_most_k_encoding(n: &BinaryNumber, k: usize) -> Clauses {
    // get the binary representation of k
    let k = to_binary(k);
    if n.bit_length < k.len() {
        // since we have a minimal representation of k, we know that in this case, n cannot be larger than k
        return Clauses::new();
    }

    let mut output: Clauses = Clauses::new();

    for i in 0..k.len() {
        // encode if k_i == 0: n_i => !n_i' for at least one i' > i where k_i' == 1
        if k[i] == false {
            let mut vars: Vec<i32> = vec![-(n.vars[i] as i32)];
            for j in i + 1..k.len() {
                if k[j] == true {
                    vars.push(-(n.vars[j] as i32))
                }
            }
            output.add_clause(Clause { vars });
        }
    }
    // we also need to make sure that if n is longer than k, that the trailing variables are obviously false
    for i in k.len()..n.bit_length {
        output.add_clause(Clause {
            vars: vec![-(n.vars[i] as i32)],
        })
    }

    return output;
}

pub fn at_most_k_exact_encoding(
    n: &BinaryNumber,
    k: usize,
    var_name_generator: &mut VarNameGenerator,
) -> (Clauses, usize) {
    // get the binary representation of k
    let k = to_binary(k);
    let mut clauses = Clauses::new();
    let out_var = var_name_generator.next();
    if n.bit_length < k.len() {
        // since we have a minimal representation of k, we know that in this case, n cannot be larger than k
        clauses.add_clause(Clause {
            vars: vec![out_var as i32],
        });
        return (clauses, out_var);
    }

    for i in 0..k.len() {
        // encode if k_i == 0: n_i => !n_i' for at least one i' > i where k_i' == 1
        let mut vars: Vec<i32> = if k[i] == true {
            vec![(n.vars[i] as i32), out_var as i32]
        } else {
            vec![-(n.vars[i] as i32), -(out_var as i32)]
        };

        for j in i + 1..k.len() {
            if k[j] == true {
                vars.push(-(n.vars[j] as i32));
            } else {
                vars.push(n.vars[j] as i32);
            }
        }
        clauses.add_clause(Clause { vars });
    }
    // we also need to make sure that if n is longer than k, that the trailing variables are obviously false
    for i in k.len()..n.bit_length {
        clauses.add_clause(Clause {
            vars: vec![-(n.vars[i] as i32)],
        })
    }

    return (clauses, out_var);
}

pub fn greater_than_logical_encoding(
    n: &BinaryNumber,
    k: usize,
    var_name_generator: &mut VarNameGenerator,
) -> (Clauses, usize) {
    let clauses = at_most_k_encoding(n, k);
    let out_var = var_name_generator.next();
    let mut out_clauses = Clauses::new();
    for mut clause in clauses.unflatten() {
        clause.vars.push(out_var as i32);
        out_clauses.add_clause(clause);
    }
    return (out_clauses, out_var);
}

pub fn greater_than_exact_logical_encoding(
    n: &BinaryNumber,
    k: usize,
    var_name_generator: &mut VarNameGenerator,
) -> (Clauses, usize) {
    let (mut clauses, leq_var) = at_most_k_exact_encoding(n, k, var_name_generator);
    let gt_var = var_name_generator.next();
    clauses.add_clause(Clause {
        vars: vec![-(gt_var as i32), -(leq_var as i32)],
    });
    clauses.add_clause(Clause {
        vars: vec![(gt_var as i32), (leq_var as i32)],
    });
    return (clauses, gt_var);
}

pub fn bounded_sum_encoding(
    n: &BinaryNumber,
    m: &BinaryNumber,
    max_bitlength: usize,
    var_name_generator: &mut VarNameGenerator,
) -> (BinaryNumber, Clauses) {
    // w.l.o.g. n >= m
    let (n, m) = if n.max > m.max { (n, m) } else { (m, n) };

    let num_output_bits: usize = number_bitlength(n.max + m.max);
    let num_output_bits: usize = max_bitlength.min(num_output_bits);

    let mut sum_bits: Vec<usize> = (0..num_output_bits)
        .map(|_| var_name_generator.next())
        .collect();
    // the number of carry bits is equal to the bitlength of the longer number
    let carry_bits: Vec<usize> = (0..n.bit_length)
        .map(|_| var_name_generator.next())
        .collect();
    let mut assertions: Clauses = Clauses::new();

    // we encode the first sum bit
    assertions.add_clause(Clause {
        vars: vec![
            -(sum_bits[0] as i32),
            (n.vars[0] as i32),
            (m.vars[0] as i32),
        ],
    });
    assertions.add_clause(Clause {
        vars: vec![
            (sum_bits[0] as i32),
            -(n.vars[0] as i32),
            (m.vars[0] as i32),
        ],
    });
    assertions.add_clause(Clause {
        vars: vec![
            (sum_bits[0] as i32),
            (n.vars[0] as i32),
            -(m.vars[0] as i32),
        ],
    });
    assertions.add_clause(Clause {
        vars: vec![
            -(sum_bits[0] as i32),
            -(n.vars[0] as i32),
            -(m.vars[0] as i32),
        ],
    });

    // we encode the first carry bit
    assertions.add_clause(Clause {
        vars: vec![-(carry_bits[0] as i32), (n.vars[0] as i32)],
    });
    assertions.add_clause(Clause {
        vars: vec![-(carry_bits[0] as i32), (m.vars[0] as i32)],
    });
    assertions.add_clause(Clause {
        vars: vec![
            (carry_bits[0] as i32),
            -(n.vars[0] as i32),
            -(m.vars[0] as i32),
        ],
    });

    for i in 1..num_output_bits {
        // remember that w.l.o.g. n >= m
        if i < m.bit_length {
            // assertions on the next sum bit
            assertions.add_clause(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    (m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    (m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    -(m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    (m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    (m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    -(m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });

            // assertions on the next carry bit
            assertions.add_clause(Clause {
                vars: vec![
                    -(carry_bits[i] as i32),
                    (n.vars[i] as i32),
                    (m.vars[i] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    -(carry_bits[i] as i32),
                    (n.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    -(carry_bits[i] as i32),
                    (m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });

            assertions.add_clause(Clause {
                vars: vec![
                    (carry_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(m.vars[i] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    (carry_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    (carry_bits[i] as i32),
                    -(m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
        } else if i < n.bit_length {
            // in this case, we only need to focus on adding n to the carry bits
            assertions.add_clause(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });

            assertions.add_clause(Clause {
                vars: vec![-(carry_bits[i] as i32), (n.vars[i] as i32)],
            });
            assertions.add_clause(Clause {
                vars: vec![-(carry_bits[i] as i32), (carry_bits[i - 1] as i32)],
            });
            assertions.add_clause(Clause {
                vars: vec![
                    (carry_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
        } else {
            // this is the case if the result is one bit longer than n, here we need to calculate the last bit
            // in this case, the last bit is equal to the previous carry bit
            sum_bits[i] = carry_bits[i - 1];
        }
    }
    let bit_length = sum_bits.len();
    // lastly, make sure that you don't cause an overflow
    if carry_bits.len() == bit_length {
        assertions.add_clause(Clause {
            vars: vec![-(carry_bits[bit_length - 1] as i32)],
        })
    }

    return (
        BinaryNumber {
            vars: sum_bits,
            max: (n.max + m.max).min((1 << max_bitlength) - 1),
            bit_length,
        },
        assertions,
    );
}

// used for testing
pub fn _equals_constant_encoding(n: &BinaryNumber, k: usize) -> Clauses {
    let k = to_binary(k);
    assert!(k.len() <= n.bit_length);
    let mut clauses: Clauses = Clauses::new();

    for i in 0..n.bit_length {
        if i < k.len() && k[i] == true {
            clauses.add_clause(Clause {
                vars: vec![(n.vars[i] as i32)],
            })
        } else {
            clauses.add_clause(Clause {
                vars: vec![-(n.vars[i] as i32)],
            })
        }
    }
    return clauses;
}

pub fn not_equals_constant_encoding(n: &BinaryNumber, k: usize) -> Clause {
    let k = to_binary(k);
    assert!(k.len() <= n.bit_length);
    let mut out: Vec<i32> = vec![];

    for i in 0..n.bit_length {
        if i < k.len() && k[i] == true {
            out.push(-(n.vars[i] as i32));
        } else {
            out.push(n.vars[i] as i32);
        }
    }
    return Clause { vars: out };
}

pub fn pairwise_encoded_at_most_one(variables: &Vec<i32>) -> Clauses {
    let mut clauses: Clauses = Clauses::new();
    for i in 0..variables.len() {
        for j in i + 1..variables.len() {
            clauses.add_clause(Clause {
                vars: vec![-variables[i], -variables[j]],
            });
        }
    }
    return clauses;
}

pub fn at_least_one_encoding(variables: Vec<i32>) -> Clause {
    return Clause { vars: variables };
}

pub fn n_equals_i_implies_m_in_j_encoding(
    n: &BinaryNumber,
    i: usize,
    m: &BinaryNumber,
    j: &Vec<usize>,
) -> Clauses {
    let mut clauses: Clauses = Clauses::new();
    // we encode n==i ==> m \in j as n!=i | m \in j
    let n_not_equals_i: Clause = not_equals_constant_encoding(&n, i);

    // we take all the binary representations of all of the numbers
    let mut binary_j: Vec<Vec<bool>> = j.iter().map(|x| to_binary(*x)).collect();

    // now we need to pad the values such that they are all the same length as m
    for i in 0..binary_j.len() {
        while binary_j[i].len() < m.bit_length {
            binary_j[i].push(false);
        }
    }

    for current_option in 0..binary_j.len() {
        // for a given option we now say that m CAN be option
        // we do this by looking at the other options m can be and asserting on the bits that uniquely identify this option
        for current_bit_depth in 0..binary_j[current_option].len() {
            let mut relevant_bit: bool = true;
            for other_option in 0..binary_j.len() {
                if binary_j[other_option].len() <= current_bit_depth {
                    continue;
                }

                if binary_j[current_option][0..current_bit_depth]
                    == binary_j[other_option][0..current_bit_depth]
                {
                    if binary_j[current_option][current_bit_depth]
                        != binary_j[other_option][current_bit_depth]
                    {
                        relevant_bit = false;
                        break;
                    } else if other_option < current_option {
                        relevant_bit = false;
                        break;
                    }
                }
            }
            if !relevant_bit {
                continue;
            }
            // now we know that the current bit uniquely identifies this option from at least one other option
            let mut clause: Vec<i32> = vec![];
            for i in 0..current_bit_depth {
                if binary_j[current_option][i] == true {
                    clause.push(-(m.vars[i] as i32))
                } else {
                    clause.push(m.vars[i] as i32)
                }
            }
            clause.push(if binary_j[current_option][current_bit_depth] == true {
                m.vars[current_bit_depth] as i32
            } else {
                -(m.vars[current_bit_depth] as i32)
            });

            clause.append(&mut n_not_equals_i.vars.clone());

            // now we have a clause ensuring the sign of this bit, we can then move on to the next bit
            clauses.add_clause(Clause { vars: clause })
        }
    }

    return clauses;
}

pub fn n_implies_m_in_j_encoding(n: usize, m: &BinaryNumber, j: &Vec<usize>) -> Clauses {
    return n_equals_i_implies_m_in_j_encoding(
        &BinaryNumber {
            vars: vec![n],
            max: 1,
            bit_length: 1,
        },
        1,
        m,
        j,
    );
}

pub fn equals_encoding(
    number1: &BinaryNumber,
    number2: &BinaryNumber,
    name_generator: &mut VarNameGenerator,
) -> (Clauses, usize) {
    let out = name_generator.next();

    let (number1, number2) = if number1.bit_length < number2.bit_length {
        (number1, number2)
    } else {
        (number2, number1)
    };

    let mut clauses = Clauses::new();
    for i in 0..number1.bit_length {
        clauses.add_clause(Clause {
            vars: vec![
                (number1.vars[i] as i32),
                -(number2.vars[i] as i32),
                -(out as i32),
            ],
        });
        clauses.add_clause(Clause {
            vars: vec![
                -(number1.vars[i] as i32),
                (number2.vars[i] as i32),
                -(out as i32),
            ],
        });
    }

    for i in number1.bit_length..number2.bit_length {
        clauses.add_clause(Clause {
            vars: vec![-(number2.vars[i] as i32), -(out as i32)],
        });
    }

    return (clauses, out);
}

pub fn exact_equals_encoding(
    number1: &BinaryNumber,
    number2: &BinaryNumber,
    name_generator: &mut VarNameGenerator,
) -> (Clauses, usize) {
    let (mut clauses, out) = equals_encoding(number1, number2, name_generator);

    let (number1, number2) = if number1.bit_length < number2.bit_length {
        (number1, number2)
    } else {
        (number2, number1)
    };

    let mut helper_vars: Vec<usize> = (0..number1.bit_length)
        .into_iter()
        .map(|_| name_generator.next())
        .collect();
    helper_vars.append(&mut number2.vars[number1.bit_length..number2.bit_length].to_vec());
    for i in 0..number1.bit_length {
        clauses.add_clause(Clause {
            vars: vec![
                -(number1.vars[i] as i32),
                -(number2.vars[i] as i32),
                -(helper_vars[i] as i32),
            ],
        });
        clauses.add_clause(Clause {
            vars: vec![
                (number1.vars[i] as i32),
                (number2.vars[i] as i32),
                -(helper_vars[i] as i32),
            ],
        });
    }

    let mut final_clause: Vec<i32> = helper_vars.into_iter().map(|x| x as i32).collect();
    final_clause.push(out as i32);

    clauses.add_clause(Clause { vars: final_clause });
    return (clauses, out);
}
