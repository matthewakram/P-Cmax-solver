use std::{fs::File, io::Write};

use crate::{encoding::encoder::Clause, common::timeout::Timeout};

pub fn _print_to_dimacs(file_name: &str, clauses: &Vec<Clause>, number_of_variables: usize, timeout: &Timeout) {
    let out = to_dimacs(clauses, number_of_variables, timeout);
    if timeout.time_finished() {
        return;
    }
    let out = out.unwrap();
    let mut file = File::create(file_name).unwrap();
    file.write(out.as_bytes()).unwrap();
}

pub fn to_dimacs(clauses: &Vec<Clause>, number_of_variables: usize, timeout: &Timeout) -> Option<String> {
    let log_2 = (usize::BITS - number_of_variables.leading_zeros()) as usize;
    let clause_size = (log_2 * clauses.len() * 4) / 3;
    let mut out = String::with_capacity(clause_size);
    out.push_str( format!("p cnf {} {}\n", number_of_variables, clauses.len()).as_str());

    for clause in clauses {
        for var in &clause.vars {
            out.push_str( &var.to_string());
            out.push_str(" ");
        }
        out.push_str(" 0\n");
        if timeout.time_finished(){
            return None;
        }
    }
    return Some(out);
}