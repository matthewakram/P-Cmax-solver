use crate::problem_instance::problem_instance::ProblemInstance;

use super::lower_bound::LowerBound;


pub struct LPT{
    
}


fn index_of_min(list: &Vec<usize>) -> usize{
    let (index, _) = list.iter().enumerate().min_by_key(|(_, x)| *x).unwrap();
    return index;
}

impl LowerBound for LPT{
    fn get_lower_bound(& self, instance: &ProblemInstance) -> usize{
        let mut solution: Vec<usize> = vec![];
        let mut total_sizes: Vec<usize> = vec![0; instance.num_processors];

        for i in 0..instance.job_sizes.len() {
            let emptiest_processor = index_of_min(&total_sizes);
            total_sizes[emptiest_processor] += instance.job_sizes[i];
            solution.push(emptiest_processor);
        }

        let makespan = *total_sizes.iter().max().unwrap();
        // the LPT algorithm has a guarantee of 4/3 + 1/3n
        let lower_bound = (makespan * 3) / 4 + 1;

        // this is interesting but never truly practical. Removing the last assigned element from each processors and calculating the makespan
        // gives a good bound. However, for the same reason that this gives a correct bound, this is also always lower than the result obtained 
        // from the PHP

        //for i in 0..instance.num_processors {
        //    let last_job = solution.iter().enumerate().filter(|(_, x)| **x == i).last();
        //    if last_job.is_none() { continue;}
        //    let (last_assigned_job, _) = last_job.unwrap();
        //    let last_assigned_job_size = instance.job_sizes[last_assigned_job];

        //    let assigned_job_weight: usize = solution.iter().enumerate().filter(|(_, x)| **x == i).map(|(i, _)| instance.job_sizes[i]).sum();

        //    lower_bound = if assigned_job_weight - last_assigned_job_size + 1 > lower_bound {assigned_job_weight - last_assigned_job_size  + 1} else {lower_bound};
        //}

        return lower_bound;
    }
}