use crate::problem_instance::partial_solution::PartialSolution;


pub struct DynNode {
    pub var: usize,
    pub aux_var: usize,
    pub range_num: usize,
    pub left_child : usize,
    pub right_child: usize,
}

pub struct Range {
    pub job_num: usize,
    pub low: usize,
    pub high: usize
}

pub fn create_range_equivalency_table(part_sol: &PartialSolution, makespan: usize) {
    let mut ranges: Vec<Vec<u32>> = vec![];
    let mut relevant_jobs = vec![];
    let mut relevant_job_sizes = vec![];
    for i in 0..part_sol.instance.num_jobs {
        if part_sol.possible_allocations[i].len() != 1 {
            relevant_jobs.push(i);
            relevant_job_sizes.push(part_sol.instance.job_sizes[i]);
        }
    }
    assert!(relevant_jobs.len() != 0);

    ranges.push(vec![3;makespan - relevant_job_sizes[relevant_job_sizes.len()-1] + 1]);
    ranges[0].append(&mut vec![2; relevant_job_sizes[relevant_job_sizes.len()-1]]);
    let mut compressed_ranges: Vec<Range> = vec![];
    compressed_ranges.push(Range { job_num: relevant_jobs[relevant_jobs.len()-1], low: 0, high: relevant_job_sizes[relevant_job_sizes.len()-1] });
    compressed_ranges.push(Range { job_num: relevant_jobs[relevant_jobs.len()-1], low: relevant_job_sizes[relevant_job_sizes.len()-1] +1, high: makespan});

    let mut current_range_num: u32 = 4;
    for relevant_job in  (0..relevant_jobs.len() -1).rev() {
        let mut last_start = makespan;
        let mut last_left = u32::MAX;
        let mut last_right = u32::MAX;
        let mut last = 0;
        let mut ranges_i = vec![0;makespan+1];
        let job_size = relevant_job_sizes[relevant_job];
        for i in (0..makespan+1).rev(){
            let left = ranges[ranges.len()-1][i];
            let right = if i + job_size <= makespan {ranges[ranges.len()-1][i + job_size]} else {u32::MAX};
            
            if last_left == left && last_right == right {
                ranges_i[i] = last;
            } else if left == right {
                last_start = i;
                last = ranges[ranges.len()-1][i];
                ranges_i[i] = last;
                last_left = left;
                last_right = right;
            } else {
                last_start = i;
                last = current_range_num;
                ranges_i[i] = last;
                last_left = left;
                last_right = right;
                current_range_num += 1;
            }
        }
        ranges.push(ranges_i);
    }

    for a in ranges {
        println!("{:?}", a);
    }
    
}