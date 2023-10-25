use super::precedence_relation_generator::{PrecedenceRelationGenerator, PrecedenceRelation};

pub struct TwoSizeReplacement {

}

impl PrecedenceRelationGenerator for TwoSizeReplacement {
    fn get_relations(&self, instance: &crate::problem_instance::problem_instance::ProblemInstance) -> Vec<PrecedenceRelation> {
        let mut out: Vec<PrecedenceRelation> = vec![];
        let mut elements: Vec<i32> = vec![-1;instance.job_sizes[0]+1];
        for i in 0..instance.num_jobs {
            elements[instance.job_sizes[i]] = i as i32;
        }

        for job1 in 0..instance.num_jobs-2 {
            for job2 in (job1+1)..instance.num_jobs {
                if elements[instance.job_sizes[job1] - instance.job_sizes[job2]] != -1 {
                    out.push(PrecedenceRelation{comes_first: job1, comes_second: vec![job2, elements[instance.job_sizes[job1] - instance.job_sizes[job2]] as usize]});
                }
            }
        }

        return out;
    }
}