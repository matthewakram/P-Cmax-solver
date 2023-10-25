use crate::precedence_relations::{precedence_relation_generator::{PrecedenceRelation, PrecedenceRelationGenerator}, size_replacement::SizeReplacement};

use super::{encoder::{OneHotEncoder, Clause, Encoder}, problem_encoding::one_hot_encoding::OneHot};



pub struct Precedence {
    pub basic: Box<dyn OneHotEncoder>,
    pub clauses: Vec<Clause>,
    precedence_relations: Vec<Box<dyn PrecedenceRelationGenerator>>,

}

impl Precedence {
    pub fn new(encoder: Box<dyn OneHotEncoder>) -> Precedence {
        return Precedence {
            basic: encoder,
            clauses: vec![],
            // TODO: have this be dynamic
            precedence_relations: vec![Box::new(SizeReplacement{}), 
            //Box::new(TwoSizeReplacement{})
            ],
        };
    }
}

impl Encoder for Precedence {
    fn basic_encode(&mut self, partial_solution: &crate::problem_instance::partial_solution::PartialSolution, makespan: usize) {
        self.basic.basic_encode(partial_solution, makespan);
        let mut clauses: Vec<Clause> = vec![];
        let precedence_relations: Vec<PrecedenceRelation> = self.precedence_relations.iter().map(|x| x.get_relations(&partial_solution.instance)).flat_map(|x| x).collect();
        
        for precedence in &precedence_relations {
            for processor in 0..partial_solution.instance.num_processors {
                
                if self.basic.get_position_var(precedence.comes_first,processor).is_some() && precedence.comes_second.iter().all(|job| self.basic.get_position_var(*job,processor).is_some()){
                    let mut future_position_vars = vec![];
                    for future_proc in processor+1..partial_solution.instance.num_processors{
                        let pos_var = self.basic.get_position_var(precedence.comes_first, future_proc);
                        if  pos_var.is_some() {
                            future_position_vars.push(-(pos_var.unwrap() as i32));
                        }
                    }
                    let comes_now_clause: Vec<i32> = precedence.comes_second.iter().map(|job | -(self.basic.get_position_var(*job,processor).unwrap() as i32)).collect();
                    for future_position_var in &future_position_vars {
                        let mut clause: Vec<i32> = comes_now_clause.clone(); 
                        clause.push(*future_position_var);
                        clauses.push(Clause { vars: clause});
                    }
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

impl OneHot for Precedence {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize>{
        return self.basic.get_position_var(job_num, proc_num);
    }
}

impl OneHotEncoder for Precedence {}