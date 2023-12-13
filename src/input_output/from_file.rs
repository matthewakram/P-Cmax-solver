use std::fs::read_to_string;

use crate::problem_instance::problem_instance::ProblemInstance;

pub fn read_from_file(file_name: &String) -> ProblemInstance {
    let binding = read_to_string(file_name).unwrap();
    let mut reader = binding.lines();
    let first_line = reader.next().unwrap();
    let first_line: Vec<&str> = first_line.split(" ").collect();
    let num_jobs = first_line[2].parse::<usize>().unwrap();
    let num_processors = first_line[3].parse::<usize>().unwrap();

    let line = reader.next().unwrap();
    let mut line: Vec<&str> = line.split(" ").collect();
    line.remove(line.len() - 1);

    let job_sizes: Vec<usize> = line
        .into_iter()
        .map(|x| x.parse::<usize>().unwrap())
        .collect();

    return ProblemInstance::new(num_processors, num_jobs, job_sizes);
}
