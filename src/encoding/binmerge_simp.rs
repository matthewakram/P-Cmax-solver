use crate::{common::timeout::Timeout, problem_instance::problem_instance::ProblemInstance};

use super::{
    binary_arithmetic, cardinality_networks,
    encoder::{Clause, Clauses, Encoder, OneHotEncoder},
    problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding},
};

#[derive(Clone)]
pub struct BinmergeSimpEncoder {
    one_hot: OneHotProblemEncoding,
    sorted: Vec<Vec<Vec<usize>>>,
    merged: Vec<Vec<Vec<usize>>>,
    clauses: Clauses,
}

impl BinmergeSimpEncoder {
    pub fn new() -> BinmergeSimpEncoder {
        return BinmergeSimpEncoder {
            one_hot: OneHotProblemEncoding::new(),
            sorted: vec![],
            merged: vec![],
            clauses: Clauses::new(),
        };
    }
}

impl Encoder for BinmergeSimpEncoder {
    // TODO add timeout to encode
    fn basic_encode(
        &mut self,
        partial_solution: &crate::problem_instance::partial_solution::PartialSolution,
        makespan: usize,
        timeout: &Timeout,
        max_num_clauses: usize,
    ) -> bool {
        self.one_hot.encode(partial_solution);
        let mut clauses: Clauses = Clauses::new();

        let false_var = self.one_hot.var_name_generator.next();
        clauses.add_clause(Clause {
            vars: vec![-(false_var as i32)],
        });
        let true_var = self.one_hot.var_name_generator.next();
        clauses.add_clause(Clause {
            vars: vec![(true_var as i32)],
        });

        let mut all_sorted: Vec<Vec<Vec<usize>>> = vec![];
        let mut all_merged: Vec<Vec<Vec<usize>>> = vec![];
        for proc in 0..partial_solution.instance.num_processors {
            if timeout.time_finished() || max_num_clauses < clauses.get_num_clauses() {
                return false;
            }
            let mut bit_level_sorted_vars: Vec<Vec<usize>> = vec![];

            // we tran now transform the condition from <= makespan to be of the form < 2^k
            let makespan_remaining = makespan - partial_solution.assigned_makespan[proc];
            if makespan_remaining == 0 {
                continue;
            }

            let diff_until_next_power_of_two = makespan_remaining.ilog2() as usize;
            let diff_until_next_power_of_two = (1 << (diff_until_next_power_of_two+1)) - makespan_remaining;
            assert!((diff_until_next_power_of_two + makespan_remaining).count_ones() == 1);
            // we now add diff_until_next_power_of_two -1 as a job that must be inserted on this processor
            let extra_job_weight = diff_until_next_power_of_two - 1;

            let makespan_bitlength = binary_arithmetic::number_bitlength(makespan_remaining + diff_until_next_power_of_two);
            //println!("proc {}", proc);
            let max_weight = partial_solution
                .instance
                .job_sizes
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    partial_solution.possible_allocations[*i].len() > 1
                        && partial_solution.possible_allocations[*i].contains(&proc)
                })
                .map(|(_, x)| x)
                .max();
            if max_weight.is_none() {
                continue;
            }
            let max_weight = (*max_weight.unwrap()).max(extra_job_weight);

            let bitlength = binary_arithmetic::number_bitlength(max_weight);

            for bit_depth in 0..bitlength {
                let relevant_jobs: Vec<usize> = partial_solution
                    .instance
                    .job_sizes
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| (*x >> bit_depth) & &0b1 == 1)
                    .filter(|(i, _)| {
                        partial_solution.possible_allocations[*i].len() > 1
                            && partial_solution.possible_allocations[*i].contains(&proc)
                    })
                    .map(|(i, _)| i)
                    .collect();

                let mut max_num_assigned: usize = 0;
                let mut total_asigned = 0;
                for i in (0..relevant_jobs.len()).rev() {
                    let relevant_job_size = partial_solution.instance.job_sizes[relevant_jobs[i]];
                    if total_asigned + relevant_job_size <= makespan_remaining {
                        total_asigned += relevant_job_size;
                        max_num_assigned += 1;
                    } else {
                        break;
                    }
                }

                //println!("relevant_job sizes {:?}", relevant_jobs.iter().map(|i| partial_solution.instance.job_sizes[*i]).collect::<Vec<usize>>());

                let mut vars: Vec<usize> = relevant_jobs
                    .iter()
                    .map(|x| self.one_hot.position_vars[*x][proc].unwrap())
                    .collect();
                if (extra_job_weight >> bit_depth) & 0b1 == 1 {
                    vars.push(true_var);
                    max_num_assigned += 1;
                }
                let (mut bitlength_clauses, sorted) = cardinality_networks::half_sort(
                    &vars,
                    max_num_assigned,
                    &mut self.one_hot.var_name_generator,
                );
                //println!("max_num_assigned {}, total_remaning {}", max_num_assigned, makespan_remaining);
                clauses.add_many_clauses(&mut bitlength_clauses);

                bit_level_sorted_vars.push(sorted);
            }

            // Now we have the sorted variables for each level, we now need to do the following
            // 1) merge the sorted vectors in order to calculate the sum
            // 2) extract the exact value of the sum from the merged levels
            // 3 assert that this sum is smaller than makespan

            let mut merge_bits: Vec<Vec<usize>> = vec![];
            merge_bits.push(bit_level_sorted_vars[0].clone());

            for bit_depth in 1..makespan_bitlength {
                let previous_carry_bits: Vec<usize> = merge_bits[bit_depth - 1]
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| i % 2 == 1)
                    .map(|(_, x)| *x)
                    .collect();

                let empty_vec = vec![];
                let (mut merge_claues, next_merge_bits) = cardinality_networks::half_merge(
                    if bit_depth < bit_level_sorted_vars.len() {
                        &bit_level_sorted_vars[bit_depth]
                    } else {
                        &empty_vec
                    },
                    &previous_carry_bits,
                    (1 << (makespan_bitlength - bit_depth)) - 1,
                    &mut self.one_hot.var_name_generator,
                );

                clauses.add_many_clauses(&mut merge_claues);
                merge_bits.push(next_merge_bits);

            }
            assert!(merge_bits[merge_bits.len() - 1].len() <=  1);
            if merge_bits.last().as_ref().unwrap().len() == 1 {
                clauses.add_clause( Clause {vars : vec![-(merge_bits[merge_bits.len() - 1][0] as i32)]});
            }
            all_merged.push(merge_bits);
            all_sorted.push(bit_level_sorted_vars);
        }

        self.merged = all_merged;
        self.sorted = all_sorted;
        self.clauses = clauses;
        //println!("{:?}", self.sorted);
        //println!("{:?}", self.merged);

        return true;
    }

    fn output(&mut self) -> Clauses {
        let mut out: Clauses = Clauses::new();
        std::mem::swap(&mut out, &mut self.clauses);
        out.add_many_clauses(&mut self.one_hot.clauses);
        return out;
    }

    fn decode(
        &self,
        instance: &ProblemInstance,
        var_assignment: &Vec<i32>,
    ) -> crate::problem_instance::solution::Solution {
        return self.one_hot.decode(instance, var_assignment);
    }

    fn get_num_vars(&self) -> usize {
        return self.one_hot.var_name_generator.peek();
    }
}

impl OneHot for BinmergeSimpEncoder {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.one_hot.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for BinmergeSimpEncoder {}
