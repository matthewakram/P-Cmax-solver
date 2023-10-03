use crate::problem_instance::problem_instance::ProblemInstance;

pub fn calc_makespan(instance: &ProblemInstance, assignment: &Vec<usize>) -> usize {
    let mut makespan = 0;

    for procesor in 0..instance.num_processors {
        makespan = makespan.max(
            assignment
                .iter()
                .enumerate()
                .filter(|(_, x)| **x == procesor)
                .map(|(i, _)| instance.job_sizes[i])
                .sum(),
        );
    }
    return makespan;
}
