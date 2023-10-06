use crate::precedence_relations::{precedence_relation_generator::{PrecedenceRelation, PrecedenceRelationGenerator}, size_replacement::SizeReplacement};

use super::{
    encoder::{Clause, Encoder}, fill_up_lite::FillUpLite,
};

pub struct FurliteWithPrecedence {
    pub basic: FillUpLite,
    pub clauses: Vec<Clause>,
    precedence_relations: Vec<Box<dyn PrecedenceRelationGenerator>>,

}

impl FurliteWithPrecedence {
    pub fn new() -> FurliteWithPrecedence {
        return FurliteWithPrecedence {
            basic: FillUpLite::new(),
            clauses: vec![],
            // TODO: have this be dynamic
            precedence_relations: vec![Box::new(SizeReplacement{})],
        };
    }
}

impl Encoder for FurliteWithPrecedence {
    fn basic_encode(&mut self, partial_solution: &crate::problem_instance::partial_solution::PartialSolution, makespan: usize) {
        self.basic.basic_encode(partial_solution, makespan);
        let mut clauses: Vec<Clause> = vec![];
        let precedence_relations: Vec<PrecedenceRelation> = self.precedence_relations.iter().map(|x| x.get_relations(&partial_solution.instance)).flat_map(|x| x).collect();
        
        for precedence in &precedence_relations {
            for processor in 0..partial_solution.instance.num_processors {
                
                if self.basic.basic.position_vars[precedence.comes_first][processor].is_some() && precedence.comes_second.iter().all(|job| self.basic.basic.position_vars[*job][processor].is_some()){
                    let mut previous_position_clauses: Vec<i32> = self.basic.basic.position_vars[precedence.comes_first].iter().enumerate().filter(|(i,x)| *i <= processor && x.is_some()).map(|(_, x)| (x.unwrap() as i32)).collect();
                    previous_position_clauses.append(&mut precedence.comes_second.iter().map(|job | -(self.basic.basic.position_vars[*job][processor].unwrap() as i32)).collect());
                    clauses.push(Clause{vars: previous_position_clauses});
                }
            }
        }

        self.clauses = clauses;
    }

    fn output(&self) -> Vec<Clause> {
        let mut out = self.clauses.clone();
        out.append(&mut self.basic.output());
        return out;
    }

    fn decode(&self, instance: &crate::problem_instance::problem_instance::ProblemInstance, solution: &Vec<i32>) -> crate::problem_instance::solution::Solution {
        return self.basic.decode(instance, solution);
    }

    fn get_num_vars(&self) -> usize {
        return self.basic.get_num_vars();
    }
}