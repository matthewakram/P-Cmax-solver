use crate::{
    common::timeout::Timeout,

    problem_instance::problem_instance::ProblemInstance, encoding::{sat_encoder::{Clauses, Encoder, Clause, OneHotEncoder}, sat_encoding::binary_arithmetic::BinaryNumber},
};

use super::{
    binary_arithmetic, cardinality_networks,

    problem_encoding::one_hot_encoding::{OneHot, OneHotProblemEncoding},
};

#[derive(Clone)]
pub struct BinmergeInterEncoder {
    one_hot: OneHotProblemEncoding,
    sorted: Vec<Vec<Vec<usize>>>,
    merged: Vec<Vec<Vec<usize>>>,
    sum_vals: Vec<Vec<usize>>,
    clauses: Clauses,
}

impl BinmergeInterEncoder {
    pub fn new() -> BinmergeInterEncoder {
        return BinmergeInterEncoder {
            one_hot: OneHotProblemEncoding::new(),
            sorted: vec![],
            merged: vec![],
            sum_vals: vec![],
            clauses: Clauses::new(),
        };
    }
}

impl Encoder for BinmergeInterEncoder {
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
                all_merged.push(vec![]);
                all_sorted.push(vec![]);
                all_sum_vals.push(vec![]);
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
                let (mut bitlength_clauses, sorted) = cardinality_networks::basic_sort(
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
                let (mut cardinality_clause, parity) = cardinality_networks::sorted_exact_parity(
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
                let (mut merge_claues, next_merge_bits) = cardinality_networks::basic_merge(
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
                    let (mut cardinality_clause, parity) =
                        cardinality_networks::sorted_exact_parity(
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

            let jobs_on_proc: Vec<usize> = partial_solution
                .instance
                .job_sizes
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    partial_solution.possible_allocations[*i].len() > 1
                        && partial_solution.possible_allocations[*i].contains(&proc)
                })
                .map(|(i, _)| i)
                .collect();

            for job in jobs_on_proc {
                let job_size = partial_solution.instance.job_sizes[job];
                let fur_val = makespan_remaining - job_size;
                if final_sum.max < fur_val {
                    continue;
                }
                let ne_fur_val_clauses: Clause =
                    binary_arithmetic::not_equals_constant_encoding(&final_sum, fur_val);
                for other_proc in proc..partial_solution.instance.num_processors {
                    if self.one_hot.position_vars[job][other_proc].is_some() {
                        let pos_var = self.one_hot.position_vars[job][other_proc].unwrap();
                        let mut ne_fur_val_clauses = ne_fur_val_clauses.clone();
                        ne_fur_val_clauses.vars.push(-(pos_var as i32));
                        clauses.add_clause(ne_fur_val_clauses);
                    }
                }
            }
        }

        assert_eq!(all_merged.len(), partial_solution.instance.num_processors);
        assert_eq!(all_sorted.len(), partial_solution.instance.num_processors);
        assert_eq!(all_sum_vals.len(), partial_solution.instance.num_processors);
        let makespan_values: Vec<BinaryNumber> = all_sum_vals
            .iter()
            .map(|x| BinaryNumber::new_from_vec(x.clone(), makespan))
            .collect();

        let mut equals_variables: Vec<Vec<Option<usize>>> = vec![];
        for proc in 0..partial_solution.instance.num_processors - 1 {
            equals_variables.push(vec![None; partial_solution.instance.num_processors]);
            for proc2 in proc + 1..partial_solution.instance.num_processors {
                let (mut equals_clauses, equals_var) = binary_arithmetic::exact_equals_encoding(
                    &makespan_values[proc],
                    &makespan_values[proc2],
                    &mut self.one_hot.var_name_generator,
                );
                clauses.add_many_clauses(&mut equals_clauses);
                equals_variables[proc][proc2] = Some(equals_var);
            }
        }

        let mut makespan_greater_than_vars: Vec<Vec<Option<usize>>> = vec![];
        for processor in 0..partial_solution.instance.num_processors - 1 {
            let remaining_makespan = makespan - partial_solution.assigned_makespan[processor];
            let mut makespan_greater_than_vars_proc: Vec<Option<usize>> =
                vec![None; remaining_makespan];
            let proc_makespan_val =
                BinaryNumber::new_from_vec(all_sum_vals[processor].clone(), remaining_makespan);
            for job in 0..partial_solution.instance.num_jobs {
                if partial_solution.is_assigned(job)
                    || self.one_hot.position_vars[job][processor].is_none()
                {
                    continue;
                }
                let max_insertion_val =
                    remaining_makespan - partial_solution.instance.job_sizes[job];

                if max_insertion_val > remaining_makespan {
                    println!("job num {} proc num {} job size {} remaining makespan {} possible assignments {:?}", job, processor, partial_solution.instance.job_sizes[job], remaining_makespan, partial_solution.possible_allocations[job]);
                }

                if makespan_greater_than_vars_proc[max_insertion_val].is_none() {
                    let (mut gt_clauses, gt) = binary_arithmetic::greater_than_logical_encoding(
                        &proc_makespan_val,
                        max_insertion_val,
                        &mut self.one_hot.var_name_generator,
                    );
                    clauses.add_many_clauses(&mut gt_clauses);
                    makespan_greater_than_vars_proc[max_insertion_val] = Some(gt);
                }

                // now I encode, if C_proc1 == C_proc2, and !(C_proc1 > max_insertion_val) => !job_on_proc2
                for processor2 in processor + 1..partial_solution.instance.num_processors {
                    if self.one_hot.position_vars[job][processor2].is_none() {
                        continue;
                    }
                    
                    clauses.add_clause(Clause {
                        vars: vec![-(equals_variables[processor][processor2].unwrap() as i32),
                        (makespan_greater_than_vars_proc[max_insertion_val].unwrap() as i32),
                        -(self.one_hot.position_vars[job][processor2].unwrap() as i32)
                        ],
                    });
                }
            }

            makespan_greater_than_vars.push(makespan_greater_than_vars_proc);
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
        return self.one_hot.decode(instance, var_assignment);
    }

    fn get_num_vars(&self) -> usize {
        return self.one_hot.var_name_generator.peek();
    }
}

impl OneHot for BinmergeInterEncoder {
    fn get_position_var(&self, job_num: usize, proc_num: usize) -> Option<usize> {
        return self.one_hot.position_vars[job_num][proc_num];
    }
}

impl OneHotEncoder for BinmergeInterEncoder {}
