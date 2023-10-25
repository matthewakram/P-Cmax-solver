
use rand::Rng;


use super::encoder::{Clause, Encoder};



pub struct RandomEncoder{
    pub basic: Box<dyn Encoder>,
    prob: f64,
}

impl RandomEncoder{
    pub fn new(encoder: Box<dyn Encoder>, prob: f64) -> RandomEncoder{
        return RandomEncoder{
            basic: encoder,
            prob: prob,
        };
    }
}

impl Encoder for RandomEncoder{
    fn basic_encode(&mut self, partial_solution: &crate::problem_instance::partial_solution::PartialSolution, makespan: usize) {
        // this name is funny because we will remove random parts of the partial solution
        let mut part_sol = partial_solution.clone();
        let mut rng: rand::rngs::ThreadRng = rand::thread_rng();

        let mut max_processor_size: Vec<usize> = vec![0; partial_solution.instance.num_processors];

        for job in 0..partial_solution.instance.num_jobs {
            for proc in &partial_solution.possible_allocations[job]{
                max_processor_size[*proc] += partial_solution.instance.job_sizes[job];
            }
        }

        let mut num_failures: usize = 0;
        loop {
            if num_failures > 3 {
                break;
            }
            let job = rng.gen_range(0..part_sol.instance.num_jobs);
            if part_sol.possible_allocations[job].len() == 1 {
                num_failures += 1;
                continue;
            }
            let proc = rng.gen_range(1..part_sol.possible_allocations[job].len());
            if max_processor_size[proc] - part_sol.instance.job_sizes[job]  <= (self.prob *  makespan as f64) as usize {
                num_failures += 1;
                continue;
            }

            // TODO: remove assigned makespan from part_sol, Not sure if it is ever used.
            let proc_num = part_sol.possible_allocations[job][proc];
            part_sol.possible_allocations[job].remove(proc);
            max_processor_size[proc_num] -= part_sol.instance.job_sizes[job];
        }

        self.basic.basic_encode(&part_sol, makespan);
    }

    fn output(&self) -> Vec<Clause> {
        return self.basic.output();
    }

    fn decode(&self, instance: &crate::problem_instance::problem_instance::ProblemInstance, solution: &Vec<i32>) -> crate::problem_instance::solution::Solution {
        return self.basic.decode(instance, solution);
    }

    fn get_num_vars(&self) -> usize {
        return self.basic.get_num_vars();
    }
}

