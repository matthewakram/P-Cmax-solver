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

pub trait IndexOf<T: Eq> {
    fn index_of(&self, elem: &T) -> Option<usize>;
}

impl<T: Eq> IndexOf<T> for Vec<T> {
    fn index_of(&self, elem: &T) -> Option<usize> {
        let a = self.into_iter().enumerate().find(|(_, x)| *x == elem);
        if a.is_none() {
            return None;
        }
        return Some(a.unwrap().0);
    }
}
