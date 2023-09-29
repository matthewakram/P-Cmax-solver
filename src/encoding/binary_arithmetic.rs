use core::num;

use super::encoder::{Clause, VarNameGenerator};

#[derive(Debug)]
pub struct BinaryNumber {
    /// the sat variable representation of a number
    pub vars: Vec<usize>,
    /// max is inclusive. The represented number can be == max
    pub max: usize,
    /// the number of bits used to represent this number
    pub bit_length: usize,
}

impl BinaryNumber {
    pub fn new(max: usize, name_generator: &mut super::encoder::VarNameGenerator) -> BinaryNumber {
        let bit_length = number_bitlength(max);
        let vars: Vec<usize> = (0..bit_length).map(|_| name_generator.next()).collect();

        return BinaryNumber {
            vars,
            max,
            bit_length,
        };
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

pub fn at_most_k_encoding(n: &BinaryNumber, k: usize) -> Vec<Clause> {
    // get the binary representation of k
    let k = to_binary(k);
    if n.bit_length < k.len() {
        // since we have a minimal representation of k, we know that in this case, n cannot be larger than k
        return vec![];
    }

    let mut output: Vec<Clause> = vec![];

    for i in 0..k.len() {
        // encode if k_i == 0: n_i => !n_i' for at least one i' > i where k_i' == 1
        if k[i] == false {
            let mut vars: Vec<i32> = vec![-(n.vars[i] as i32)];
            for j in i + 1..k.len() {
                if k[j] == true {
                    vars.push(-(n.vars[j] as i32))
                }
            }
            output.push(Clause { vars });
        }
    }
    // we also need to make sure that if n is longer than k, that the trailing variables are obviously false
    for i in k.len()..n.bit_length {
        output.push(Clause {
            vars: vec![-(n.vars[i] as i32)],
        })
    }

    return output;
}

//TODO: TEST THIS
pub fn bounded_sum_encoding(
    n: &BinaryNumber,
    m: &BinaryNumber,
    max_bitlength: usize,
    var_name_generator: &mut VarNameGenerator,
) -> (BinaryNumber, Vec<Clause>) {
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
    let mut assertions: Vec<Clause> = vec![];

    // we encode the first sum bit
    assertions.push(Clause {
        vars: vec![
            -(sum_bits[0] as i32),
            (n.vars[0] as i32),
            (m.vars[0] as i32),
        ],
    });
    assertions.push(Clause {
        vars: vec![
            (sum_bits[0] as i32),
            -(n.vars[0] as i32),
            (m.vars[0] as i32),
        ],
    });
    assertions.push(Clause {
        vars: vec![
            (sum_bits[0] as i32),
            (n.vars[0] as i32),
            -(m.vars[0] as i32),
        ],
    });
    assertions.push(Clause {
        vars: vec![
            -(sum_bits[0] as i32),
            -(n.vars[0] as i32),
            -(m.vars[0] as i32),
        ],
    });

    // we encode the first carry bit
    assertions.push(Clause {
        vars: vec![-(carry_bits[0] as i32), (n.vars[0] as i32)],
    });
    assertions.push(Clause {
        vars: vec![-(carry_bits[0] as i32), (m.vars[0] as i32)],
    });
    assertions.push(Clause {
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
            assertions.push(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    (m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    (m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    -(m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    (m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    (m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    -(m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });

            // assertions on the next carry bit
            assertions.push(Clause {
                vars: vec![
                    -(carry_bits[i] as i32),
                    (n.vars[i] as i32),
                    (m.vars[i] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    -(carry_bits[i] as i32),
                    (n.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    -(carry_bits[i] as i32),
                    (m.vars[i] as i32),
                    (carry_bits[i - 1] as i32),
                ],
            });

            assertions.push(Clause {
                vars: vec![
                    (carry_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(m.vars[i] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    (carry_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    (carry_bits[i] as i32),
                    -(m.vars[i] as i32),
                    -(carry_bits[i - 1] as i32),
                ],
            });
        } else if i < n.bit_length {
            // in this case, we only need to focus on adding n to the carry bits
            assertions.push(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    (carry_bits[i] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    (carry_bits[i] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    (sum_bits[i] as i32),
                    (n.vars[i] as i32),
                    -(carry_bits[i] as i32),
                ],
            });
            assertions.push(Clause {
                vars: vec![
                    -(sum_bits[i] as i32),
                    -(n.vars[i] as i32),
                    -(carry_bits[i] as i32),
                ],
            });

            // we encode the first carry bit
            assertions.push(Clause {
                vars: vec![-(carry_bits[i] as i32), (n.vars[i] as i32)],
            });
            assertions.push(Clause {
                vars: vec![-(carry_bits[i] as i32), (carry_bits[i - 1] as i32)],
            });
            assertions.push(Clause {
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

    // lastly, make sure that you don't cause an overflow
    if carry_bits.len() == max_bitlength {
        assertions.push(Clause {
            vars: vec![-(carry_bits[max_bitlength - 1] as i32)],
        })
    }
    let bit_length = sum_bits.len();
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
pub fn equals_constant_encoding(n: &BinaryNumber, k: usize) -> Vec<Clause> {
    let k = to_binary(k);
    assert!(k.len() <= n.bit_length);
    let mut clauses: Vec<Clause> = vec![];

    for i in 0..n.bit_length {
        if i < k.len() && k[i] == true {
            clauses.push(Clause {
                vars: vec![(n.vars[i] as i32)],
            })
        } else {
            clauses.push(Clause {
                vars: vec![-(n.vars[i] as i32)],
            })
        }
    }
    return clauses;
}

//used for testing
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

pub fn pairwise_encoded_at_most_one(variables: &Vec<i32>) -> Vec<Clause> {
    let mut clauses: Vec<Clause> = vec![];
    for i in 0..variables.len() {
        for j in i + 1..variables.len() {
            clauses.push(Clause {
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
) -> Vec<Clause> {
    let mut clauses: Vec<Clause> = vec![];
    // we encode n==i ==> m \in j as n!=i | m \in j
    let n_not_equals_i: Clause = not_equals_constant_encoding(&n, i);

    // we take all the binary representations of all of the numbers
    let mut binary_j: Vec<Vec<bool>> = j.iter().map(|x| to_binary(*x)).collect();
    
    // now we need to pad the values such that they are all the same length as m
    for i in 0..binary_j.len() {
        while binary_j[i].len() < m.bit_length{
            binary_j[i].push(false);
        }
    }

    println!("{:?}", binary_j);

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
                println!("irrelavant: {} {}", current_option, current_bit_depth );
                continue;
            }
            // now we know that the current bit uniquely identifies this option from at least one other option
            let mut clause: Vec<i32> = vec![];
            for i in 0..current_bit_depth {
                if binary_j[current_option][i] == true {
                    clause.push(-(m.vars[i] as i32))
                } else {
                    clause.push((m.vars[i] as i32))
                }
            }
            clause.push(if binary_j[current_option][current_bit_depth] == true {
                m.vars[current_bit_depth] as i32
            } else {
                -(m.vars[current_bit_depth] as i32)
            });

            clause.append(&mut n_not_equals_i.vars.clone());

            // now we have a clause ensuring the sign of this bit, we can then move on to the next bit
            clauses.push(Clause { vars: clause })
        }
    }

    return clauses;
}
