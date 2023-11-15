use super::precedence_relation_generator::{PrecedenceRelationGenerator, PrecedenceRelation};



#[derive(Clone)]
pub struct SizeReplacement {

}

impl PrecedenceRelationGenerator for SizeReplacement {
    fn get_relations(&self, instance: &crate::problem_instance::problem_instance::ProblemInstance) -> Vec<PrecedenceRelation> {
        let mut out: Vec<PrecedenceRelation> = vec![];
        for job1 in 0..instance.num_jobs {
            for job2 in job1+1..instance.num_jobs {
                if instance.job_sizes[job1] != instance.job_sizes[job2] {
                    break;
                }
                out.push(PrecedenceRelation{comes_first: job1, comes_second: vec![job2]});
            }
        }
        return out;
    }
}