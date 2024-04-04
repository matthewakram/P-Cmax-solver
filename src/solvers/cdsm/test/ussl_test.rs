#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rand::{distributions::Uniform, Rng};

    use crate::solvers::cdsm::{state_db::StateDB, ussl::USSL};

    #[test]
    pub fn ussl_smoke_test() {
        let mut ls = USSL::new(4, 100, 3, 10);

        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, 20);
        let v: Vec<Vec<u16>> = (0..10000)
            .map(|_| (0..4).map(|_| rng.sample(&range)).collect::<Vec<u16>>())
            .collect();
        let start = Instant::now();

        for i in 0..10000 {
            ls.insert_list(&v[i]);
            if !ls.is_present(&v[i]) {
                println!("{:?}", ls);
                println!("{:?}", v[i]);
            }
            assert!(ls.is_present(&v[i]))
        }
        println!("{:?}", ls);
        println!("time {}", start.elapsed().as_secs_f64());

    }
}
