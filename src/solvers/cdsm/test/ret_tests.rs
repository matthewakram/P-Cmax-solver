#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rand::{distributions::Uniform, Rng};

    use crate::solvers::cdsm::{list_set::MultiListSet, ret::RET};

    #[test]
    pub fn ret_smoke_test(){
        let job_sizes: Vec<usize> = vec![11, 7, 5, 3, 2];
        let ret = RET::new(&job_sizes, 13);

        println!("{:?}", ret);
    }

}