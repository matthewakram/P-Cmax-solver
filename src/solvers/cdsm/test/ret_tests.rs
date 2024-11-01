#[cfg(test)]
mod tests {

    use crate::solvers::cdsm::ret::RET;

    #[test]
    pub fn ret_smoke_test() {
        let job_sizes: Vec<usize> = vec![11, 7, 5, 3, 2];
        let ret = RET::new(&job_sizes, 13, job_sizes.len() - 1, 1000);

        println!("{:?}", ret);
    }
}
