#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rand::{distributions::Uniform, Rng};

    use crate::solvers::cdsm::list_set::MultiListSet;

    #[test]
    pub fn simple_insert(){
        let mut ls = MultiListSet::new(10, 100, 5);

        let v1 = vec![15, 0, 9, 8, 7];
        let v2 = vec![15, 0, 9, 9, 7];
        ls.insert_list(&v1);
        assert!(ls.is_present(&v1));
        assert!(!ls.is_present(&v2));
    }

    #[test]
    pub fn random_insert(){
        let mut ls = MultiListSet::new(300, 225, 32);

        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, 225);
        let mut vectors = vec![];
        let mut num_fails = 0;
        for i in 0..1000000 {
            let v: Vec<u16> = (0..32).map(|_| rng.sample(&range)).collect();

            if vectors.contains(&v) {
                assert!(ls.is_present(&v));
            } else {
                assert!(!ls.is_present(&v));
                if ls.insert_list(&v){
                    vectors.push(v);
                }else {
                    num_fails += 1;
                }
            }
        }
        println!("num fails {}", num_fails);
    }

    #[test]
    pub fn random_time_insert(){
        let mut ls = MultiListSet::new(300, 225, 50);

        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, 225);
        let v: Vec<Vec<u16>> = (0..1000000).map(|_| (0..32).map(|_| rng.sample(&range)).collect::<Vec<u16>>()).collect();
        let start = Instant::now();
        for i in 0..1000000 {

            if !ls.is_present(&v[i]){
                ls.insert_list(&v[i]);
            }
        }
        println!("time {}", start.elapsed().as_secs_f64());
    }

    #[test]
    pub fn random_clear(){
        let mut ls = MultiListSet::new(10, 20, 10);

        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, 20);
        let v: Vec<Vec<u16>> = (0..1000).map(|_| (0..10).map(|_| rng.sample(&range)).collect::<Vec<u16>>()).collect();
        for i in 0..1000 {

            if !ls.is_present(&v[i]){
                ls.insert_list(&v[i]);
            }
        }

        println!("{}", ls);

        ls.clear_table(3);
        println!("{}", ls);

        ls.clear_all();
        println!("{}", ls);
    }

}