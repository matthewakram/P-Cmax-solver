use super::encoder::{Encoder, VarNameGenerator};



pub struct BasicEncoder {
    var_name_generator: VarNameGenerator,
}

impl BasicEncoder {
    pub fn new() -> BasicEncoder{
        return BasicEncoder { var_name_generator: VarNameGenerator::new() }
    }
}


impl Encoder for BasicEncoder {

    fn basic_encode(&mut self, partial_solution: &crate::problem_instance::partial_solution::PartialSolution) {
        // here the presence of a variable indicates that process i can be put on server j
        let mut position_variables: Vec<Vec<Option<usize>>> = vec![];
        for i in 0..partial_solution.instance.num_jobs {
            let mut position_vars_i: Vec<Option<usize>> = vec![];
            for j in 0..partial_solution.instance.num_processors {
                if partial_solution.possible_allocations[i].contains(&j) {
                    position_vars_i.push(Some(self.var_name_generator.next()));
                } else {
                    position_vars_i.push(None)
                }
            }
            position_variables.push(position_vars_i);
        }

        //TODO
    }

    fn output(&self) {
        todo!()
    }

    fn decode(&self) -> crate::problem_instance::solution::Solution {
        todo!()
    }
}