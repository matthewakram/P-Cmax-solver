use std::{fs::File, io::Write};

use crate::encoding::encoder::Clause;

pub fn print_to_dimacs(file_name: &str, clauses: Vec<Clause>, number_of_variables: usize) {
    let mut file = File::create(file_name).unwrap();
    file.write(format!("p cnf {} {}\n", number_of_variables, clauses.len()).as_bytes()).unwrap();
    for clause in clauses {
        let mut string: String = String::new();
        for var in clause.vars {
            string += &var.to_string();
            string += " ";
        }
        string += " 0\n";
        file.write(string.as_bytes()).unwrap();
    }
}