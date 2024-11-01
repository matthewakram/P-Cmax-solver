use crate::{
    common::timeout::Timeout,
    encoding::sat_encoder::{Clause, Clauses, Encoder, OneHotEncoder},
    precedence_relations::{
        precedence_relation_generator::{PrecedenceRelation, PrecedenceRelationGenerator},
        size_replacement::SizeReplacement,
        two_size_replacement::TwoSizeReplacement,
    },
};

use super::problem_encoding::one_hot_encoding::OneHot;

#[derive(Clone)]
pub struct Precedence {
    pub basic: Box<dyn OneHotEncoder>,
    pub clauses: Clauses,
    precedence_relations: Vec<Box<dyn PrecedenceRelationGenerator>>,
}

unsafe impl Send for Precedence {}

impl Precedence {
    pub fn new(encoder: Box<dyn OneHotEncoder>, num_precs: usize) -> Precedence {
        let mut precs: Vec<Box<dyn PrecedenceRelationGenerator>> = vec![];
        if num_precs >= 1 {
            precs.push(Box::new(SizeReplacement {}));
        }
        if num_precs >= 2 {
            precs.push(Box::new(TwoSizeReplacement { limit: 1000 }))
        }
        return Precedence {
            basic: encoder,
            clauses: Clauses::new(),
            // TODO: have this be dynamic
            precedence_relations: precs,
        };
    }
}

impl Encoder for Precedence {
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
        timeout: &Timeout,
        max_num_clauses: usize,
    ) -> bool {
        let success =
            self.basic
                .basic_encode(partial_solution, makespan, &timeout, max_num_clauses);
        if !success {
            return false;
        }
        let mut clauses: Clauses = Clauses::new();
        let precedence_relations: Vec<PrecedenceRelation> = self
            .precedence_relations
            .iter()
            .map(|x| x.get_relations(&partial_solution.instance))
            .flat_map(|x| x)
            .collect();

        for precedence in &precedence_relations {
            for processor in 0..partial_solution.instance.num_processors {
                if self
                    .basic
                    .get_position_var(precedence.comes_first, processor)
                    .is_some()
                    && precedence
                        .comes_second
                        .iter()
                        .all(|job| self.basic.get_position_var(*job, processor).is_some())
                {
                    let mut future_position_vars = vec![];
                    for future_proc in processor + 1..partial_solution.instance.num_processors {
                        let pos_var = self
                            .basic
                            .get_position_var(precedence.comes_first, future_proc);
                        if pos_var.is_some() {
                            future_position_vars.push(-(pos_var.unwrap() as i32));
                        }
                    }
                    let comes_now_clause: Vec<i32> = precedence
                        .comes_second
                        .iter()
                        .map(|job| -(self.basic.get_position_var(*job, processor).unwrap() as i32))
                        .collect();
                    for future_position_var in &future_position_vars {
                        let mut clause: Vec<i32> = comes_now_clause.clone();
                        clause.push(*future_position_var);
                        clauses.add_clause(Clause { vars: clause });
                    }
                }
            }
            if timeout.time_finished() {
                return false;
            }
        }

        self.clauses = clauses;
        return true;
    }

    fn output(&mut self) -> Clauses {
        let mut out: Clauses = Clauses::new();
        std::mem::swap(&mut out, &mut self.clauses);
        out.add_many_clauses(&mut self.basic.output());
        return out;
    }

    fn decode(
        &self,
        instance: &crate::problem_instance::problem_instance::ProblemInstance,
        solution: &Vec<i32>,
    ) -> crate::problem_instance::solution::Solution {
        return self.basic.decode(instance, solution);
    }

    fn get_num_vars(&self) -> usize {
        return self.basic.get_num_vars();
    }
}

impl OneHot for Precedence {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.basic.get_position_var(job_num, proc_num);
    }
}

impl OneHotEncoder for Precedence {}
