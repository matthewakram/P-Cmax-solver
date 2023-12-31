use crate::encoding::sat_encoder::{Clause, VarNameGenerator, Clauses};


pub fn basic_sort(
    vars: &Vec<usize>,
    max_true: usize,
    var_name_generator: &mut VarNameGenerator,
) -> (Clauses, Vec<usize>) {
    let mut clauses: Clauses = Clauses::new();

    if vars.is_empty() {
        //println!("empty");
        return (clauses, vec![]);
    }
    let mut previous_vars = vec![vars[0]];
    for i in 1..vars.len() {
        let mut new_vars: Vec<usize> = vec![];
        for j in 0..(i).min(max_true) + 1 {
            let new_var = var_name_generator.next();
            new_vars.push(new_var);
            if j > 0 {
                clauses.add_clause(Clause {
                    vars: vec![
                        -(previous_vars[j - 1] as i32),
                        -(vars[i] as i32),
                        new_var as i32,
                    ],
                });
                // TODO: see performance
                clauses.add_clause(Clause {
                    vars: vec![previous_vars[j - 1] as i32, -(new_var as i32)],
                });
            } else {
                clauses.add_clause(Clause {
                    vars: vec![-(previous_vars[j] as i32), new_var as i32],
                });
                clauses.add_clause(Clause {
                    vars: vec![-(vars[i] as i32), new_var as i32],
                });
            }

            if j < previous_vars.len() {
                clauses.add_clause(Clause {
                    vars: vec![
                        (previous_vars[j] as i32),
                        (vars[i] as i32),
                        -(new_var as i32),
                    ],
                });
                clauses.add_clause(Clause {
                    vars: vec![-(previous_vars[j] as i32), (new_var as i32)],
                });
            } else {
                clauses.add_clause(Clause {
                    vars: vec![vars[i] as i32, -(new_var as i32)],
                })
            }
        }
        //println!("{:?}", new_vars);
        previous_vars = new_vars;
    }
    if previous_vars.len() >= max_true + 1 {
        assert_eq!(previous_vars.len(), max_true + 1);
        //println!("outlen {}, max_true {}", previous_vars.len(), max_true);
        clauses.add_clause(Clause {
            vars: vec![-(previous_vars[previous_vars.len() - 1] as i32)],
        });
        previous_vars.pop();
    }
    return (clauses, previous_vars);
}

pub fn half_sort(
    vars: &Vec<usize>,
    max_true: usize,
    var_name_generator: &mut VarNameGenerator,
) -> (Clauses, Vec<usize>) {
    let mut clauses = Clauses::new();
    if vars.len() <= 1 {
        return (clauses, vars.clone());
    } else if vars.len() == 2 {
        return half_merge(&vec![vars[0]], &vec![vars[1]], max_true, var_name_generator);
    }

    let half_size = vars.len() / 2;
    let lower_half: Vec<usize> = vars
        .iter()
        .enumerate()
        .filter(|(i, _)| *i < half_size)
        .map(|(_, x)| *x)
        .collect();
    let upper_half: Vec<usize> = vars
        .iter()
        .enumerate()
        .filter(|(i, _)| *i >= half_size)
        .map(|(_, x)| *x)
        .collect();

    let (mut clauses_lower, sorted_lower) = half_sort(&lower_half, max_true, var_name_generator);
    let (mut clauses_upper, sorted_upper) = half_sort(&upper_half, max_true, var_name_generator);

    clauses.add_many_clauses(&mut clauses_lower);
    clauses.add_many_clauses(&mut clauses_upper);

    let (mut clauses_merge, sorted_vars) =
        half_merge(&sorted_lower, &sorted_upper, max_true, var_name_generator);
    clauses.add_many_clauses(&mut clauses_merge);

    return (clauses, sorted_vars);
}

pub fn basic_merge(
    vars1: &Vec<usize>,
    vars2: &Vec<usize>,
    max_true: usize,
    var_name_generator: &mut VarNameGenerator,
) -> (Clauses, Vec<usize>) {
    let mut clauses = Clauses::new();
    if vars1.is_empty() {
        if vars2.len() > max_true {
            clauses.add_clause(Clause {
                vars: vec![-(vars2[max_true] as i32)],
            });
        }
        return (clauses, vars2.clone());
    } else if vars2.is_empty() {
        if vars1.len() > max_true {
            clauses.add_clause(Clause {
                vars: vec![-(vars1[max_true] as i32)],
            });
        }
        return (clauses, vars1.clone());
    }
    let merged_size = (vars1.len() + vars2.len()).min(max_true + 1);
    let mut merged: Vec<usize> = vec![];
    for _ in 0..merged_size {
        merged.push(var_name_generator.next());
    }

    for i in 0..merged_size {
        if i < vars1.len() {
            clauses.add_clause(Clause {
                vars: vec![-(vars1[i] as i32), (merged[i] as i32)],
            })
        }
        if i < vars2.len() {
            clauses.add_clause(Clause {
                vars: vec![-(vars2[i] as i32), (merged[i] as i32)],
            })
        }
    }

    for i in 0..(max_true).min(vars1.len()) {
        for j in 0..(max_true - i).min(vars2.len()) {
            clauses.add_clause(Clause {
                vars: vec![
                    -(vars1[i] as i32),
                    -(vars2[j] as i32),
                    (merged[i + j + 1] as i32),
                ],
            });
            clauses.add_clause(Clause {
                vars: vec![
                    (vars1[i] as i32),
                    (vars2[j] as i32),
                    -(merged[i + j] as i32),
                ],
            });
        }
    }

    if merged.len() == max_true + 1 {
        clauses.add_clause(Clause {
            vars: vec![-(merged[max_true] as i32)],
        });
        merged.pop();
    }

    return (clauses, merged);
}

pub fn half_merge(
    vars1: &Vec<usize>,
    vars2: &Vec<usize>,
    max_true: usize,
    var_name_generator: &mut VarNameGenerator,
) -> (Clauses, Vec<usize>) {
    assert!(max_true > 0);
    let mut clauses = Clauses::new();
    if vars1.len() == 1 && vars2.len() == 1 {
        let mut out_vars: Vec<usize> = vec![var_name_generator.next(), var_name_generator.next()];
        clauses.add_clause(Clause {
            vars: vec![-(vars1[0] as i32), -(vars2[0] as i32), (out_vars[1] as i32)],
        });
        clauses.add_clause(Clause {
            vars: vec![-(vars1[0] as i32), (out_vars[0] as i32)],
        });
        clauses.add_clause(Clause {
            vars: vec![-(vars2[0] as i32), (out_vars[0] as i32)],
        });
        if max_true == 1 {
            clauses.add_clause(Clause {
                vars: vec![-(out_vars.pop().unwrap() as i32)],
            });
        }
        return (clauses, out_vars);
    } else if vars1.len() == 0 {
        if vars2.len() > max_true {
            clauses.add_clause(Clause {
                vars: vec![-(vars2[max_true] as i32)],
            });
            return (clauses, vars2[0..max_true + 1].to_vec());
        } else {
            return (clauses, vars2.clone());
        }
    } else if vars2.len() == 0 {
        if vars1.len() > max_true {
            clauses.add_clause(Clause {
                vars: vec![-(vars1[max_true] as i32)],
            });
            return (clauses, vars1[0..max_true + 1].to_vec());
        } else {
            return (clauses, vars1.clone());
        }
    }

    let vars1_evens: Vec<usize> = vars1
        .iter()
        .enumerate()
        .filter(|(i, _)| *i % 2 == 0)
        .map(|(_, x)| *x)
        .collect();
    let vars1_odds: Vec<usize> = vars1
        .iter()
        .enumerate()
        .filter(|(i, _)| *i % 2 == 1)
        .map(|(_, x)| *x)
        .collect();
    let vars2_evens: Vec<usize> = vars2
        .iter()
        .enumerate()
        .filter(|(i, _)| *i % 2 == 0)
        .map(|(_, x)| *x)
        .collect();
    let vars2_odds: Vec<usize> = vars2
        .iter()
        .enumerate()
        .filter(|(i, _)| *i % 2 == 1)
        .map(|(_, x)| *x)
        .collect();

    let (mut even_clauses, merged_evens) =
        half_merge(&vars1_evens, &vars2_evens, max_true, var_name_generator);
    let (mut odd_clauses, merged_odds) =
        half_merge(&vars1_odds, &vars2_odds, max_true, var_name_generator);

    clauses.add_many_clauses(&mut even_clauses);
    clauses.add_many_clauses(&mut odd_clauses);

    let mut merged_vars = vec![];
    merged_vars.push(merged_evens[0]);

    let mut i = 0;
    while merged_vars.len() <= max_true {
        if i + 1 < merged_evens.len() && i < merged_odds.len() {
            let c = var_name_generator.next();
            if merged_vars.len() % 2 == 0 {
                // if we are at an odd index

                clauses.add_clause(Clause {
                    vars: vec![
                        -(merged_evens[i + 1] as i32),
                        -(merged_odds[i] as i32),
                        c as i32,
                    ],
                });
                i += 1;
            } else {
                // if we are at an even index;

                clauses.add_clause(Clause {
                    vars: vec![-(merged_evens[i + 1] as i32), (c as i32)],
                });
                clauses.add_clause(Clause {
                    vars: vec![-(merged_odds[i] as i32), (c as i32)],
                });
            }
            merged_vars.push(c);
        } else if i + 1 < merged_evens.len() {
            merged_vars.push(merged_evens[i + 1]);
            i += 1;
        } else if i < merged_odds.len() {
            merged_vars.push(merged_odds[i]);
            i += 1;
        } else {
            break;
        }
    }

    if merged_vars.len() == max_true + 1 {
        clauses.add_clause(Clause {
            vars: vec![-(merged_vars.pop().unwrap() as i32)],
        });
    }
    return (clauses, merged_vars);
}

pub fn sorted_parity(
    vars: &Vec<usize>,
    var_name_generator: &mut VarNameGenerator,
) -> (Clauses, usize) {
    let mut clauses = Clauses::new();

    let parity = var_name_generator.next();

    let mut i = 0;
    while i < vars.len() - 1 {
        clauses.add_clause(Clause {
            vars: vec![-(vars[i] as i32), vars[i + 1] as i32, parity as i32],
        });
        i += 2;
    }

    if i == vars.len() - 1 {
        clauses.add_clause(Clause {
            vars: vec![-(vars[i] as i32), parity as i32],
        });
    }

    return (clauses, parity);
}

pub fn sorted_exact_parity(
    vars: &Vec<usize>,
    var_name_generator: &mut VarNameGenerator,
) -> (Clauses, usize) {
    let mut clauses = Clauses::new();

    let parity = var_name_generator.next();

    let mut i = 0;
    while i < vars.len() - 1 {
        if i % 2 == 0 {
            clauses.add_clause(Clause {
                vars: vec![-(vars[i] as i32), vars[i + 1] as i32, parity as i32],
            });
        } else {
            clauses.add_clause(Clause {
                vars: vec![-(vars[i] as i32), vars[i + 1] as i32, -(parity as i32)],
            })
        }
        i += 1;
    }

    if i % 2 == 0 {
        clauses.add_clause(Clause {
            vars: vec![-(vars[i] as i32), parity as i32],
        });
    } else {
        clauses.add_clause(Clause {
            vars: vec![-(vars[i] as i32), -(parity as i32)],
        });
    }

    return (clauses, parity);
}
