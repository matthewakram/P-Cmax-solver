use crate::{common::timeout::Timeout, problem_instance::problem_instance::ProblemInstance, encoding::sat_encoder::{Clauses, Encoder, Clause, OneHotEncoder}};

use super::{
    binary_arithmetic, cardinality_networks,
    problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding},
};

#[derive(Clone)]
pub struct BinmergeEncoder {
    one_hot: OneHotProblemEncoding,
    sorted: Vec<Vec<Vec<usize>>>,
    merged: Vec<Vec<Vec<usize>>>,
    sum_vals: Vec<Vec<usize>>,
    clauses: Clauses,
}

impl BinmergeEncoder {
    pub fn new() -> BinmergeEncoder {
        return BinmergeEncoder {
            one_hot: OneHotProblemEncoding::new(),
            sorted: vec![],
            merged: vec![],
            sum_vals: vec![],
            clauses: Clauses::new(),
        };
    }
}

/// This is binmerge, but the PBC is not normalised
impl Encoder for BinmergeEncoder {
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

        let mut all_sorted: Vec<Vec<Vec<usize>>> = vec![];
        let mut all_merged: Vec<Vec<Vec<usize>>> = vec![];
        let mut all_sum_vals: Vec<Vec<usize>> = vec![];
        for proc in 0..partial_solution.instance.num_processors {
            if timeout.time_finished() || max_num_clauses < clauses.get_num_clauses() {
                return false;
            }
            let mut bit_level_sorted_vars: Vec<Vec<usize>> = vec![];
            let makespan_remaining = makespan - partial_solution.assigned_makespan[proc];
            let makespan_bitlength = binary_arithmetic::number_bitlength(makespan_remaining);
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
            let max_weight = *max_weight.unwrap();

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

                let vars: Vec<usize> = relevant_jobs
                    .iter()
                    .map(|x| self.one_hot.position_vars[*x][proc].unwrap())
                    .collect();
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
            let mut sum_val: Vec<usize> = vec![];
            if merge_bits[0].is_empty() {
                sum_val.push(false_var);
            } else {
                let (mut cardinality_clause, parity) = cardinality_networks::sorted_parity(
                    &merge_bits[0],
                    &mut self.one_hot.var_name_generator,
                );
                sum_val.push(parity);
                clauses.add_many_clauses(&mut cardinality_clause);
            }

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

                if merge_bits[bit_depth].is_empty() {
                    sum_val.push(false_var);
                } else {
                    let (mut cardinality_clause, parity) = cardinality_networks::sorted_parity(
                        &merge_bits[bit_depth],
                        &mut self.one_hot.var_name_generator,
                    );
                    sum_val.push(parity);
                    clauses.add_many_clauses(&mut cardinality_clause);
                }
            }
            all_merged.push(merge_bits);
            all_sorted.push(bit_level_sorted_vars);
            all_sum_vals.push(sum_val.clone());

            let final_sum =
                binary_arithmetic::BinaryNumber::new_from_vec(sum_val, makespan_remaining);
            clauses.add_many_clauses(&mut binary_arithmetic::at_most_k_encoding(
                &final_sum,
                makespan_remaining,
            ));
        }

        self.merged = all_merged;
        self.sorted = all_sorted;
        self.sum_vals = all_sum_vals;
        self.clauses = clauses;
        //println!("{:?}", self.sorted);
        //println!("{:?}", self.merged);
        //println!("{:?}", self.sum_vals);

        return true;
    }

    fn output(&mut self) -> Clauses {
        let mut out: Clauses = Clauses::new();
        std::mem::swap(&mut out, &mut self.clauses);
        out.add_many_clauses(&mut self.one_hot.clauses);
        //input_output::to_dimacs::_print_to_dimacs("./test", out.clone(), self.get_num_vars(), &Timeout::new(10.0));
        //let num_vars = self.get_num_vars();
        //for i in &out {
        //    for v in &i.vars{
        //        if v.abs() as usize > num_vars {
        //           //println!("error occured at {} {}", v, num_vars);
        //        }
        //        assert!(v.abs() as usize <= num_vars);
        //    }
        //}
        return out;
    }

    fn decode(
        &self,
        instance: &ProblemInstance,
        var_assignment: &Vec<i32>,
    ) -> crate::problem_instance::solution::Solution {
        //println!("assignment {:?}", var_assignment);
        // for proc in 0..self.merged.len() {
        //    //println!("sorted_vals, proc {}", proc);
        //     for i in 0..self.sorted[proc].len() {
        //         let mut values = vec![];
        //         for var in &self.sorted[proc][i] {
        //             if var_assignment.contains(&(*var as i32)) {
        //                 values.push(1);
        //             } else {
        //                 values.push(0);
        //             }
        //         }
        //        //println!("{:?}", values);
        //
        //     }
        //    //println!("merged_vals, proc {}", proc);
        //     for i in 0..self.merged[proc].len() {
        //         let mut values = vec![];
        //         for var in &self.merged[proc][i] {
        //             if var_assignment.contains(&(*var as i32)) {
        //                 values.push(1);
        //             } else {
        //                 values.push(0);
        //             }
        //         }
        //        //println!("{:?}", values);
        //
        //     }
        // }
        return self.one_hot.decode(instance, var_assignment);
    }

    fn get_num_vars(&self) -> usize {
        return self.one_hot.var_name_generator.peek();
    }
}

impl OneHot for BinmergeEncoder {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.one_hot.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for BinmergeEncoder {}
