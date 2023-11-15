use dyn_clone::DynClone;

use crate::problem_instance::problem_instance::ProblemInstance;




pub trait PrecedenceRelationGenerator: DynClone {
    fn get_relations(&self, instance: &ProblemInstance)-> Vec<PrecedenceRelation>;
}

dyn_clone::clone_trait_object!(PrecedenceRelationGenerator);

pub struct PrecedenceRelation{
    pub comes_first : usize,
    pub comes_second: Vec<usize>
}