use std::{fs::File, io::Write};

use crate::{common::timeout::Timeout, encoding::encoder::Clauses};

pub fn _print_to_dimacs(
    file_name: &str,
    clauses: Clauses,
    number_of_variables: usize,
    timeout: &Timeout,
) {
    let out = to_dimacs(clauses, number_of_variables, timeout);
    if timeout.time_finished() {
        return;
    }
    let out = out.unwrap();
    let mut file = File::create(file_name).unwrap();
    file.write(out.as_bytes()).unwrap();
}

pub fn to_dimacs(
    clauses: Clauses,
    number_of_variables: usize,
    timeout: &Timeout,
) -> Option<String> {
    let log_2 = (usize::BITS - number_of_variables.leading_zeros()) as usize;
    let clause_size = log_2 * clauses.len();
    let mut out = String::with_capacity(clause_size);
    out.push_str(
        format!(
            "p cnf {} {}\n",
            number_of_variables,
            clauses.get_num_clauses()
        )
        .as_str(),
    );

    let mut num_chars_done: usize = 0;
    for var in clauses.iter() {
        out.push_str(&var.to_string());
        out.push_str(" ");
        num_chars_done += 1;

        if num_chars_done == 10000 {
            if timeout.time_finished() {
                return None;
            }
            num_chars_done = 0;
        }
    }
    return Some(out);
}
