
#[derive(Debug)]
pub struct RET {
    data: Vec<u16>,
    makespan: usize,
    decreased_by: usize,
}

impl RET {
    fn calc_index(&self, job: usize, u: usize) -> usize {
        assert!(u + self.decreased_by <= self.makespan);
        return (self.makespan+1) * job + u + self.decreased_by;
    }

    fn right(
        data: &Vec<u16>,
        jobs_sizes: &Vec<usize>,
        makespan: usize,
        job_index: usize,
        u: usize,
    ) -> u16 {
        assert_ne!(job_index, jobs_sizes.len() - 1);
        let offset = u + jobs_sizes[job_index];
        if offset > makespan {
            return u16::MAX;
        }
        let index = (job_index + 1) * (makespan+1) + offset;
        return data[index];
    }

    fn left(
        data: &Vec<u16>,
        _jobs_sizes: &Vec<usize>,
        makespan: usize,
        job_index: usize,
        u: usize,
    ) -> u16 {
        let offset = u;
        let index = (job_index + 1) * (makespan+1) + offset;
        return data[index];
    }

    pub fn new(jobs_sizes: &Vec<usize>, makespan: usize) -> RET {
        assert!((makespan as u16) < u16::MAX);

        let num_jobs = jobs_sizes.len();
        let decreased_by = 0;

        let mut data: Vec<u16> = vec![0; (makespan+1) * num_jobs];

        for u in ((num_jobs - 1) * (makespan+1))..(data.len() - jobs_sizes[jobs_sizes.len()-1]) {
            data[u] = 1;
        }


        let mut data_pointer = ((num_jobs - 1) * (makespan+1)) -1;
        for job_index in (0..num_jobs-1).rev() {
            let mut range: u16 = 0;
            let mut last_left: u16 = 0;
            let mut last_right: u16 = u16::MAX;
            for u in (0..makespan+1).rev() {
                let l = Self::left(&data, &jobs_sizes, makespan, job_index, u);
                let r = Self::right(&data, &jobs_sizes, makespan, job_index, u);

                if last_left != l || last_right != r {
                    range += 1;
                }

                data[data_pointer] = range;
                if data_pointer == 0 {
                    break;
                }
                data_pointer -= 1;
                last_left = l;
                last_right = r;
            }
        }

        return RET { data, makespan, decreased_by };
    }

    pub fn get_range(&self, job_num: usize, u: usize) -> u16 {
        let index = self.calc_index(job_num, u);
        return self.data[index];
    }

    pub fn decrease_makespan_to(&mut self, new_makespan: usize) {
        assert!(new_makespan < self.makespan);
        self.decreased_by = self.makespan - new_makespan;
    }
}
