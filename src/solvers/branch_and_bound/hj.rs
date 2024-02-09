use crate::{
    bdd::compressed_ret::CompressedRet,
    common::common::IndexOf,
    problem_instance::{
        partial_solution::PartialSolution, problem_instance::ProblemInstance, solution::Solution,
    },
    problem_simplification::{
        fill_up_rule::FillUpRule, final_simp_rule::FinalizeRule, half_size_rule::HalfSizeRule,
        simplification_rule::SimpRule,
    },
    solvers::solver_manager::SolverManager,
};

#[derive(Clone)]
pub struct HJ {
    fur_rule: bool,
}

fn subset_sum(
    remaining_jobs: &Vec<usize>,
    instance: &ProblemInstance,
    goal: usize,
    lower_bound: usize,
) -> (Vec<usize>, usize) {
    let mut dp: Vec<i16> = vec![-1; goal.max(lower_bound) + 1];

    for job in remaining_jobs {
        let job = *job;
        let job_size = instance.job_sizes[job];
        if dp[job_size] == -1 {
            dp[job_size] = job as i16;
        }
        for pos in 1..(dp.len() - job_size) {
            if dp[pos] != -1 && dp[pos] != job as i16 {
                if dp[job_size + pos] == -1 {
                    dp[job_size + pos] = job as i16;
                }
            }
        }
        if dp[goal] != -1 {
            break;
        }
    }

    let mut pointer: usize = dp.len() - 1;

    while dp[pointer] == -1 {
        assert_ne!(pointer, 0);
        pointer -= 1;
    }

    let reached_val = pointer;

    let mut calculated_val = 0;
    let mut result: Vec<usize> = vec![];
    while pointer != 0 {
        let job = dp[pointer];
        assert_ne!(job, -1);
        result.push(job as usize);
        pointer -= instance.job_sizes[job as usize];
        calculated_val += instance.job_sizes[job as usize];
    }
    assert_eq!(calculated_val, reached_val);
    return (result, reached_val);
}

impl HJ {
    pub fn new() -> HJ {
        return HJ {
            fur_rule: true,
        };
    }

    pub fn new_base() -> HJ {
        return HJ {
            fur_rule: false,
        };
    }

    fn solve_rec(
        &self,
        instance: &ProblemInstance,
        // we maintain the for part_sol.makespan < upper
        part_sol: &mut PartialAssignment,
        ret: &mut CompressedRet,
        lower: usize,
        best_makespan_found: usize,
        processor_to_assign_to: usize,
        timeout: &crate::common::timeout::Timeout,
    ) -> Result<Option<PartialAssignment>, ()> {
        if timeout.time_finished() {
            return Result::Err(());
        }
        part_sol.num_nodes_explored += 1;
        let lower: usize = lower.max(part_sol.makespan);
        if best_makespan_found <= lower {
            return Ok(None);
        }
        assert!(part_sol.makespan < best_makespan_found);
        // TODO: incorporate this part into lower
        let remaining_space: usize = best_makespan_found
            - 1
            - part_sol.makespans[processor_to_assign_to]
            + (instance.num_processors - processor_to_assign_to - 1) * (best_makespan_found - 1);

        if part_sol.min_space_required > remaining_space {
            return Ok(None);
        }

        if part_sol.unassigned.len() == 0 {
            assert!(part_sol.makespan < best_makespan_found);
            let next_makespan_to_check = part_sol.makespan;
            if (self.fur_rule) && next_makespan_to_check >= instance.job_sizes[0]
            {
                *ret = CompressedRet::new(
                    &(0..instance.num_jobs).into_iter().collect(),
                    &instance.job_sizes,
                    next_makespan_to_check,
                );
            } else {
                *ret = CompressedRet::new(
                    &(0..0).into_iter().collect(),
                    &(0..0).into_iter().collect(),
                    0,
                );
            }
            return Ok(Some(part_sol.clone()));
        }
        // ======================================
        let remaining_makespan =
            best_makespan_found - 1 - part_sol.makespans[processor_to_assign_to];

        if instance.job_sizes[*part_sol.unassigned.last().unwrap()] > remaining_makespan {
            return self.solve_rec(
                instance,
                part_sol,
                ret,
                lower,
                best_makespan_found,
                processor_to_assign_to + 1,
                timeout,
            );
        }

        // TODO:
        // if there are only two processors remaining then we can use SSS
        if best_makespan_found < 10000 && processor_to_assign_to == instance.num_processors - 2 {
            let current_makespan = part_sol.makespan;
            let mut makespan_to_test_for_last_two_procs = current_makespan;
            while makespan_to_test_for_last_two_procs < best_makespan_found {
                let (sss_solution, makespan) = subset_sum(
                    &part_sol.unassigned,
                    instance,
                    makespan_to_test_for_last_two_procs,
                    lower,
                );
                if part_sol.min_space_required - makespan <= makespan_to_test_for_last_two_procs {
                    // best solution found!
                    let mut solution = part_sol.clone();
                    //println!("sol makespan after before first assing {}, ", solution.makespan);
                    assert_eq!(part_sol.makespans[instance.num_processors - 1], 0);
                    for job in sss_solution {
                        solution.assign(job, instance.num_processors - 2, instance);
                    }
                    //println!("sol makespan after first assign {}, reprted {}, ", solution.makespan, makespan);
                    while !solution.unassigned.is_empty() {
                        solution.assign(
                            solution.unassigned[0],
                            instance.num_processors - 1,
                            instance,
                        );
                    }
                    //println!("sol makespan after rest assign {}", solution.makespan);
                    let next_makespan_to_check = solution.makespan;
                    if (self.fur_rule)
                        && next_makespan_to_check >= instance.job_sizes[0]
                    {
                        *ret = CompressedRet::new(
                            &(0..instance.num_jobs).into_iter().collect(),
                            &instance.job_sizes,
                            next_makespan_to_check,
                        );
                    } else {
                        *ret = CompressedRet::new(
                            &(0..0).into_iter().collect(),
                            &(0..0).into_iter().collect(),
                            0,
                        );
                    }
                    return Ok(Some(solution));
                }
                makespan_to_test_for_last_two_procs += 1;
            }

            return Ok(None);
        }

        let mut largest_fitting = 0;
        while instance.job_sizes[part_sol.unassigned[largest_fitting]] > remaining_makespan {
            largest_fitting += 1;
        }

        // FUR RULE
        let fur_job = part_sol.unassigned[largest_fitting];
        if self.fur_rule
            && ret.are_same_range(
                fur_job,
                part_sol.makespans[processor_to_assign_to],
                best_makespan_found - 1 - instance.job_sizes[fur_job],
            )
        {
            if part_sol.rejection_makespan[fur_job][processor_to_assign_to] != usize::MAX {
                // we have already decided to not put this job on this processor. Due to the FUR rule, blah blah blah
                // TODO: not sure about this but probably
                return Ok(None);
            }
            part_sol.assign(fur_job, processor_to_assign_to, instance);
            let sol = self.solve_rec(
                instance,
                part_sol,
                ret,
                lower,
                best_makespan_found,
                processor_to_assign_to,
                timeout,
            );
            if sol.is_err() {
                return Err(());
            }
            part_sol.unassign(fur_job, instance);
            let sol = sol.unwrap();
            if sol.is_some() {
                // if sol is found, then we know that we can improve upon best_makespan_found, but because we used the
                // FUR, we dont know if we can improve this solution even further without the FUR
                let sol = sol.unwrap();
                assert!(sol.makespan < best_makespan_found);
                let best_makespan_found: usize = sol.makespan;
                // if the optiml solution has been reached, or we know that this assignment is not the reason that the makespan is so large,
                // then we know we cannot achieve a better solution here
                if best_makespan_found <= lower
                    || part_sol.makespans[processor_to_assign_to] + instance.job_sizes[fur_job]
                        < sol.makespan
                {
                    return Ok(Some(sol));
                }

                // now we have to revert the FUR decision, and recurse
                let better_sol = self.solve_rec(
                    instance,
                    part_sol,
                    ret,
                    lower,
                    best_makespan_found,
                    processor_to_assign_to,
                    timeout,
                );
                if better_sol.is_err() {
                    return Err(());
                }
                let better_sol = better_sol.unwrap();
                if better_sol.is_none() {
                    //println!("happened");
                    return Ok(Some(sol));
                } else {
                    return Ok(better_sol);
                }
            } else {
                // here sol is None, so we know we cannot improve upon best_makespan_found
                return Ok(None);
            }
        }

        let currently_unassigned = part_sol.unassigned.clone();
        let mut best_sol = None;
        let mut best_makespan_found = best_makespan_found;
        let mut prev_job_size = usize::MAX;
        // BRANCHING
        for job in currently_unassigned {
            let remaining_makespan =
                best_makespan_found - 1 - part_sol.makespans[processor_to_assign_to];
            if instance.job_sizes[job] > remaining_makespan {
                continue;
            }

            if instance.job_sizes[job] == prev_job_size {
                part_sol.reject(job, processor_to_assign_to);
                continue;
            }
            prev_job_size = instance.job_sizes[job];

            if part_sol.rejection_makespan[job][processor_to_assign_to] != usize::MAX {
                continue;
            }

            part_sol.assign(job, processor_to_assign_to, instance);
            let sol = self.solve_rec(
                instance,
                part_sol,
                ret,
                lower,
                best_makespan_found,
                processor_to_assign_to,
                timeout,
            );
            part_sol.unassign(job, instance);
            if sol.is_err() {
                return Err(());
            }

            let sol = sol.unwrap();
            if sol.is_some() {
                let sol = sol.unwrap();
                best_makespan_found = sol.makespan;
                best_sol = Some(sol);
            }
            part_sol.reject(job, processor_to_assign_to);
        }
        // unreject all jobs rejected in the previous loop
        let unassigned = part_sol.unassigned.clone();
        for job in unassigned {
            if part_sol.rejection_makespan[job][processor_to_assign_to]
                == part_sol.makespans[processor_to_assign_to]
            {
                part_sol.undo_reject(job, processor_to_assign_to);
            }
        }

        return Ok(best_sol);
    }
}

impl SolverManager for HJ {
    fn solve(
        &mut self,
        instance: &crate::problem_instance::problem_instance::ProblemInstance,
        lower: usize,
        upper: &crate::problem_instance::solution::Solution,
        timeout: &crate::common::timeout::Timeout,
        _verbose: bool,
    ) -> Option<crate::problem_instance::solution::Solution> {
        let makespan_to_test = upper.makespan - 1;
        let partial_solution = PartialSolution::new(instance.clone());

        let mut hsr = HalfSizeRule {};
        let mut fur: FillUpRule = FillUpRule {};
        let mut finalize: FinalizeRule = FinalizeRule {};
        let partial_solution: PartialSolution =
            hsr.simplify(&partial_solution, makespan_to_test).unwrap();
        let partial_solution: PartialSolution =
            fur.simplify(&partial_solution, makespan_to_test).unwrap();
        let partial_solution = finalize.simplify(&partial_solution, makespan_to_test);
        if partial_solution.is_none() {
            return Some(upper.clone());
        }
        let mut part_sol = PartialAssignment::new(instance);

        let ret = &mut CompressedRet::new(
            &(0..instance.num_jobs).into_iter().collect(),
            &instance.job_sizes,
            makespan_to_test,
        );

        let sol = self.solve_rec(
            instance,
            &mut part_sol,
            ret,
            lower,
            upper.makespan,
            0,
            timeout,
        );
        if sol.is_err() {
            return None;
        }
        let sol = sol.unwrap();
        if sol.is_none() {
            return Some(upper.clone());
        } else {
            let sol = sol.unwrap();
            let sol = Solution {
                makespan: sol.makespan,
                assignment: sol.assignment,
            };
            return Some(sol);
        }
    }
}

#[derive(Clone)]
struct PartialAssignment {
    pub assignment: Vec<usize>,
    pub makespans: Vec<usize>,
    pub unassigned: Vec<usize>,
    pub makespan: usize,
    pub min_space_required: usize,
    pub time_point_assigned: Vec<usize>,
    pub num_nodes_explored: usize,
    /// The makespan at which each job was rejecteed from being assigned here. rejection_makespan[job][proc]
    pub rejection_makespan: Vec<Vec<usize>>,
}
impl PartialAssignment {
    pub fn new(instance: &ProblemInstance) -> PartialAssignment {
        let assignment: Vec<usize> = vec![usize::MAX; instance.num_jobs];
        let makespans: Vec<usize> = vec![0; instance.num_processors];
        let unassigned: Vec<usize> = (0..instance.num_jobs).collect();
        let makespan = 0;
        let min_space_required: usize = instance.job_sizes.iter().sum();
        let time_point_assigned: Vec<usize> = vec![0; instance.num_jobs];
        return PartialAssignment {
            assignment,
            makespans,
            unassigned,
            makespan,
            min_space_required,
            time_point_assigned,
            num_nodes_explored: 0,
            rejection_makespan: vec![vec![usize::MAX; instance.num_processors]; instance.num_jobs],
        };
    }

    pub fn reject(&mut self, job: usize, proc: usize) {
        self.rejection_makespan[job][proc] = self.makespans[proc];
    }

    pub fn undo_reject(&mut self, job: usize, proc: usize) {
        self.rejection_makespan[job][proc] = usize::MAX;
    }

    pub fn assign(&mut self, job: usize, proc: usize, instance: &ProblemInstance) {
        assert!(self.assignment[job] == usize::MAX);
        self.assignment[job] = proc;
        self.time_point_assigned[job] = self.makespans[proc];
        self.makespans[proc] += instance.job_sizes[job];
        self.makespan = self.makespan.max(self.makespans[proc]);
        let job_pos = self.unassigned.index_of(&job).unwrap();
        self.unassigned.remove(job_pos);
        self.min_space_required -= instance.job_sizes[job];
    }

    pub fn unassign(&mut self, job: usize, instance: &ProblemInstance) {
        assert!(self.assignment[job] != usize::MAX);
        let proc = self.assignment[job];
        self.time_point_assigned[job] = 0;
        self.assignment[job] = usize::MAX;
        let old_makespan = self.makespans[proc];
        self.makespans[proc] -= instance.job_sizes[job];

        if old_makespan == self.makespan {
            self.makespan = *self.makespans.iter().max().unwrap();
        }

        let mut index = 0;
        while index < self.unassigned.len() && self.unassigned[index] < job {
            index += 1;
        }
        self.unassigned.insert(index, job);
        self.min_space_required += instance.job_sizes[job];
    }
}
