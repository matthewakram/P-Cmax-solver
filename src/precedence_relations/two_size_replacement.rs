use super::precedence_relation_generator::{PrecedenceRelationGenerator, PrecedenceRelation};

pub struct TwoSizeReplacement {
    pub limit: usize,
}

impl PrecedenceRelationGenerator for TwoSizeReplacement {
    fn get_relations(&self, instance: &crate::problem_instance::problem_instance::ProblemInstance) -> Vec<PrecedenceRelation> {
        let mut out: Vec<PrecedenceRelation> = vec![];


        let mut total_num = 0;
        for job1 in 0..instance.num_jobs-2 {
            for job2 in (job1+1)..instance.num_jobs-1 {
                if total_num >= self.limit {
                    println!("tooo many relations");
                    return out;
                }
                for job3 in (job2+1)..instance.num_jobs {
                    if instance.job_sizes[job3] + instance.job_sizes[job2] == instance.job_sizes[job1]{
                        total_num += 1;
                        out.push(PrecedenceRelation { comes_first: job1, comes_second: vec![job2, job3] })
                    }
                }
            }
        }

        return out;
    }
}