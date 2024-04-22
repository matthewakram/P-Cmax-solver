
#[derive(Debug)]
pub struct RET {
    data: Vec<u16>,
    makespan: usize,
    decreased_by: usize,
    last_relevant_index: usize,
    index_of_first_represented_job: usize,
}

impl RET {
    fn calc_index(&self, job: usize, u: usize) -> usize {
        assert!(u + self.decreased_by <= self.makespan);
        return (self.makespan + 1) * job + u + self.decreased_by;
    }

    pub fn get_space_consuption(&self) -> usize{
        return self.data.len() * 8;
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
        let index = (job_index + 1) * (makespan + 1) + offset;
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
        let index = (job_index + 1) * (makespan + 1) + offset;
        return data[index];
    }

    pub fn new(jobs_sizes: &Vec<usize>, makespan: usize, last_relevant_index: usize, mem_limit: usize) -> RET {
        if jobs_sizes.len() == 0 {
            return RET {
                data: vec![],
                makespan,
                decreased_by: 0,
                last_relevant_index,
                index_of_first_represented_job: 0,
            };
        }
        assert!((makespan as u16) < u16::MAX);

        let num_jobs = jobs_sizes.len();
        let decreased_by = 0;

        let num_levels = (mem_limit / ((makespan+1) * std::mem::size_of::<u16>())) + 1;
        let num_levels = num_levels.min(num_jobs);
        let index_of_first_represented_job = num_jobs- num_levels;
        let mut data: Vec<u16> = vec![0; (makespan + 1) * num_levels];
        

        // println!("index of first job {}, last relevant index{}" , index_of_first_represented_job, last_relevant_index);
        for u in ((last_relevant_index - index_of_first_represented_job) * (makespan+1))..((last_relevant_index+1 - index_of_first_represented_job)* (makespan+1) - jobs_sizes[last_relevant_index]) {
            data[u] = 1;
        }


        let mut data_pointer = (( last_relevant_index - index_of_first_represented_job) * (makespan+1)) -1;
        for job_index in (index_of_first_represented_job..last_relevant_index).rev() {
            let mut range: u16 = 0;
            let mut last_left: u16 = 0;
            let mut last_right: u16 = u16::MAX;
            for u in (0..makespan + 1).rev() {
                let l = Self::left(&data, &jobs_sizes, makespan, job_index - index_of_first_represented_job, u);
                let r = Self::right(&data, &jobs_sizes, makespan,  job_index - index_of_first_represented_job, u);

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

        return RET {
            data,
            makespan,
            decreased_by,
            last_relevant_index,
            index_of_first_represented_job,
        };
    }

    pub fn get_range(&self, job_num: usize, u: usize) -> u16 {
        assert!(job_num <= self.last_relevant_index);
        if job_num < self.index_of_first_represented_job {
            return u as u16;
        }
        let index = self.calc_index(job_num - self.index_of_first_represented_job, u);
        return self.data[index];
    }

    pub fn is_relevant(&self, job_num: usize) -> bool {
        return job_num <= self.last_relevant_index;
    }

    pub fn decrease_makespan_to(
        &mut self,
        jobs_sizes: &Vec<usize>,
        new_makespan: usize,
        new_last_relevant_index: usize,
    ) {
        assert!(new_makespan < self.makespan);
        self.decreased_by = self.makespan - new_makespan;

        if self.last_relevant_index < new_last_relevant_index {
            self.last_relevant_index = new_last_relevant_index;

            for u in ((self.last_relevant_index - self.index_of_first_represented_job) * (self.makespan+1))..((self.last_relevant_index+1 - self.index_of_first_represented_job)* (self.makespan+1) - jobs_sizes[self.last_relevant_index]) {
                self.data[u] = 1;
            }
    
    
            let mut data_pointer = ((self.last_relevant_index - self.index_of_first_represented_job) * (self.makespan+1)) -1;
            for job_index in (self.index_of_first_represented_job..self.last_relevant_index).rev() {
                let mut range: u16 = 0;
                let mut last_left: u16 = 0;
                let mut last_right: u16 = u16::MAX;
                let mut changed = false;
                for u in (0..self.makespan + 1).rev() {
                    let l = Self::left(&self.data, &jobs_sizes, self.makespan, job_index - self.index_of_first_represented_job, u);
                    let r = Self::right(&self.data, &jobs_sizes, self.makespan, job_index - self.index_of_first_represented_job, u);
    
                    if last_left != l || last_right != r {
                        range += 1;
                    }

                    if self.data[data_pointer] != range {
                        self.data[data_pointer] = range;
                        changed = true;
                    }

                    if data_pointer == 0 {
                        break;
                    }
                    data_pointer -= 1;
                    last_left = l;
                    last_right = r;
                }
                if changed == false {
                    break;
                }
            }
        }
    }
}
