use crate::problem_instance::problem_instance::ProblemInstance;




pub trait PrecedenceRelationGenerator {
    fn get_relations(&self, instance: &ProblemInstance)-> Vec<PrecedenceRelation>;
}

pub struct PrecedenceRelation{
    pub comes_first : usize,
    pub comes_second: Vec<usize>
}