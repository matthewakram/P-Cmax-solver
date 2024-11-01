#[derive(Debug)]
pub struct CompressedRet {
    ranges: Vec<usize>,
}

impl CompressedRet {
    pub fn new(jobs: &Vec<usize>, weights: &Vec<usize>, makespan: usize) -> CompressedRet {
        let mut ranges = vec![usize::MAX; makespan];
        for i in (0..jobs.len()).rev() {
            let job = jobs[i];
            let job_size = weights[i];

            if ranges[makespan - job_size] == usize::MAX {
                ranges[makespan - job_size] = job;
            }
            for u in (0..makespan - job_size).rev() {
                if ranges[u] == usize::MAX {
                    if ranges[u + job_size] != usize::MAX && ranges[u + job_size] != job {
                        ranges[u] = job;
                    }
                }
            }
        }

        // for i in ranges.clone() {
        // if i == usize::MAX {
        // print!(", -1");
        // } else {
        // print!(", {}", i);
        // }
        // }
        // println!();
        return CompressedRet { ranges };
    }

    pub fn are_same_range(&self, job: usize, u1: usize, u2: usize) -> bool {
        let (u1, u2) = if u1 > u2 { (u2, u1) } else { (u1, u2) };

        for i in u1..u2 {
            if self.ranges[i] != usize::MAX && self.ranges[i] >= job {
                //println!("AAAAAAAAAAAAAA");
                return false;
            }
        }
        return true;
    }
}
