use std::collections::HashMap;

use bitvec::bitvec;

use crate::{
    bdd::bdd_dyn::RangeTable,
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
pub struct BranchAndBoundLogging {
    stats: HashMap<String, f64>,
    fur_rule: bool,
    inter_rule: bool,
}

impl BranchAndBoundLogging {
    pub fn new() -> BranchAndBoundLogging {
        return BranchAndBoundLogging {
            stats: HashMap::new(),
            fur_rule: true,
            inter_rule: true,
        };
    }

    pub fn new_basic() -> BranchAndBoundLogging {
        return BranchAndBoundLogging {
            stats: HashMap::new(),
            fur_rule: false,
            inter_rule: false,
        };
    }

    pub fn new_inter() -> BranchAndBoundLogging {
        return BranchAndBoundLogging {
            stats: HashMap::new(),
            fur_rule: false,
            inter_rule: true,
        };
    }
}

fn min_procs(part_sol: &PartialAssignment) -> (usize, usize) {
    let mut min = usize::MAX;
    let mut second_min = usize::MAX;
    let mut second_min_proc = usize::MAX;
    let mut min_proc = usize::MAX;
    for i in 0..part_sol.makespans.len() {
        if part_sol.makespans[i] < min {
            second_min = min;
            second_min_proc = min_proc;
            min = part_sol.makespans[i];
            min_proc = i;
        } else if part_sol.makespans[i] < second_min {
            second_min = part_sol.makespans[i];
            second_min_proc = i;
        }
    }
    return (min_proc, second_min_proc);
}

impl BranchAndBoundLogging {
    fn solve_rec(
        &self,
        instance: &ProblemInstance,
        // we maintain the for part_sol.makespan < upper
        part_sol: &mut PartialAssignment,
        ret: &mut RangeTable,
        lower: usize,
        best_makespan_found: usize,
        timeout: &crate::common::timeout::Timeout,
        log: &mut Vec<u16>,
    ) -> Result<Option<PartialAssignment>, ()> {
        if timeout.time_finished() {
            return Result::Err(());
        }
        let lower: usize = lower.max(part_sol.makespan);
        if best_makespan_found <= lower {
            return Ok(None);
        }
        assert!(part_sol.makespan < best_makespan_found);
        // TODO: incorporate this part into lower
        let remaining_space: usize = part_sol
            .makespans
            .iter()
            .map(|x| best_makespan_found - 1 - *x)
            .filter(|x| x >= &instance.job_sizes[*part_sol.unassigned.last().unwrap()])
            .sum();

        if part_sol.min_space_required > remaining_space {
            return Ok(None);
        }
        // ======================================

        if part_sol.unassigned.len() == 3 {
            let mut first_part_sol = part_sol.clone();
            let (min_proc, second_min_proc) = min_procs(&first_part_sol);
            first_part_sol.assign(
                first_part_sol.unassigned[0],
                second_min_proc,
                instance,
                false,
            );
            first_part_sol.assign(first_part_sol.unassigned[0], min_proc, instance, false);
            first_part_sol.assign(first_part_sol.unassigned[0], min_proc, instance, false);
            let first_option_makespan = first_part_sol.makespan;
            assert_eq!(first_part_sol.unassigned.len(), 0);

            let mut second_part_sol = part_sol.clone();
            second_part_sol.assign(second_part_sol.unassigned[0], min_proc, instance, false);
            let (min_proc, _) = min_procs(&second_part_sol);
            second_part_sol.assign(second_part_sol.unassigned[0], min_proc, instance, false);
            let (min_proc, _) = min_procs(&second_part_sol);
            second_part_sol.assign(second_part_sol.unassigned[0], min_proc, instance, false);
            let second_option_makespan = second_part_sol.makespan;
            assert_eq!(second_part_sol.unassigned.len(), 0);

            let better_option = if first_option_makespan < second_option_makespan {
                first_part_sol
            } else {
                second_part_sol
            };
            if better_option.makespan < best_makespan_found {
                //println!("found solution with makesapan {}", better_option.makespan);
                let next_makespan_to_check = better_option.makespan - 1;
                if self.fur_rule || self.inter_rule {
                    *ret = RangeTable::new(
                        &(0..instance.num_jobs).into_iter().collect(),
                        &instance.job_sizes,
                        next_makespan_to_check,
                    );
                }

                return Ok(Some(better_option));
            } else {
                return Ok(None);
            }
        }

        let level = part_sol.unassigned[0];
        let mut ranges: Vec<usize> = part_sol
            .makespans
            .iter()
            .map(|x| ret.get_range(level, *x).unwrap())
            .collect();
        ranges.sort();
        ranges.push(best_makespan_found - 1);
        ranges.push(level);
        for i in 0..ranges.len() {
            log.push(ranges[i] as u16);
        }

        let mut fur_job = usize::MAX;
        let mut fur_proc = usize::MAX;
        if self.fur_rule {
            for proc in 0..instance.num_processors {
                let proc_makespan = part_sol.makespans[proc];
                if best_makespan_found - proc_makespan
                    < instance.job_sizes[*part_sol.unassigned.last().unwrap()]
                {
                    continue;
                }
                for i in 0..part_sol.unassigned.len() {
                    if best_makespan_found - proc_makespan
                        > instance.job_sizes[part_sol.unassigned[i]]
                        && (i + 1 == part_sol.unassigned.len()
                            || instance.job_sizes[part_sol.unassigned[i]]
                                != instance.job_sizes[part_sol.unassigned[i + 1]])
                    {
                        let job = part_sol.unassigned[i];
                        if ret.get_range(job, best_makespan_found - 1 - instance.job_sizes[job])
                            == ret.get_range(job, proc_makespan)
                        {
                            fur_job = job;
                            fur_proc = proc;
                        }
                        break;
                    }
                }
            }
        }

        if fur_job != usize::MAX {
            // in this case we use the FUR and recurse

            part_sol.assign(fur_job, fur_proc, instance, true);
            let sol = self.solve_rec(
                instance,
                part_sol,
                ret,
                lower,
                best_makespan_found,
                timeout,
                log,
            );
            if sol.is_err() {
                return Err(());
            }
            part_sol.unassign(fur_job, instance, true);
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
                    || part_sol.makespans[fur_proc] + instance.job_sizes[fur_job] < sol.makespan
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
                    timeout,
                    log,
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

        // if we reach this point, we know we cannot use the FUR rule here. Thus our only option is to branch
        let job_to_branch_on = part_sol.unassigned[0];

        // In order to direct the search towards solutions more quickly, we sort the procs in decreasing order of available makespan
        let mut best_sol: Option<PartialAssignment> = None;
        let mut best_makespan_found = best_makespan_found;

        let job_to_branch_on_size = instance.job_sizes[job_to_branch_on];
        let mut precedence = 0;
        // TODO: Fix this
        if job_to_branch_on > 0 && instance.job_sizes[job_to_branch_on - 1] == job_to_branch_on_size
        {
            assert!(!part_sol.fur_assignments[job_to_branch_on - 1]);
            precedence = part_sol.time_point_assigned[job_to_branch_on - 1];
        }

        let mut procs: Vec<(usize, usize)> = part_sol
            .makespans
            .iter()
            .enumerate()
            .filter(|(proc_num, makespan)| {
                best_makespan_found - **makespan > instance.job_sizes[job_to_branch_on]
                    && part_sol.makespans[*proc_num] >= precedence
            })
            .map(|(x, a)| (x, *a))
            .collect();
        procs.sort_by(|(_, makespan1), (_, makespan2)| makespan1.cmp(makespan2));
        let num_unassigned: usize = part_sol.unassigned.len();
        let procs_to_branch_on: Vec<usize> = procs
            .into_iter()
            .map(|(proc, _)| proc)
            .take(num_unassigned)
            .collect();

        let mut last_range: usize = usize::MAX;

        for proc in procs_to_branch_on {
            if best_makespan_found - part_sol.makespans[proc]
                <= instance.job_sizes[job_to_branch_on]
            {
                continue;
            }
            let range = if self.inter_rule {
                ret.get_range(job_to_branch_on, part_sol.makespans[proc])
                    .unwrap()
            } else {
                part_sol.makespans[proc]
            };
            if last_range == range {
                continue;
            }
            last_range = range;

            part_sol.assign(job_to_branch_on, proc, instance, false);
            let sol = self.solve_rec(
                instance,
                part_sol,
                ret,
                lower,
                best_makespan_found,
                timeout,
                log,
            );

            if sol.is_err() {
                return Err(());
            }
            part_sol.unassign(job_to_branch_on, instance, false);

            let sol = sol.unwrap();
            if sol.is_some() {
                let sol = sol.unwrap();
                best_makespan_found = sol.makespan;
                best_sol = Some(sol);
                if best_makespan_found <= lower {
                    return Ok(best_sol);
                }
                last_range = if self.inter_rule {
                    ret.get_range(job_to_branch_on, part_sol.makespans[proc])
                        .unwrap()
                } else {
                    part_sol.makespans[proc]
                };
            }
        }

        return Ok(best_sol);
    }
}

impl SolverManager for BranchAndBoundLogging {
    fn get_stats(&self) -> HashMap<String, f64> {
        return self.stats.clone();
    }

    fn solve(
        &mut self,
        instance: &crate::problem_instance::problem_instance::ProblemInstance,
        lower: usize,
        upper: &crate::problem_instance::solution::Solution,
        timeout: &crate::common::timeout::Timeout,
        _verbose: bool,
    ) -> Option<crate::problem_instance::solution::Solution> {
        let mem_size_key = "mem_used".to_owned();
        let makespan_to_test = upper.makespan - 1;
        self.stats.insert(
            mem_size_key,
            ((makespan_to_test * instance.num_jobs
                + instance.num_jobs * 4
                + instance.num_processors)
                * 8) as f64,
        );
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

        let ret = &mut RangeTable::new(
            &(0..instance.num_jobs).into_iter().collect(),
            &instance.job_sizes,
            makespan_to_test,
        );

        let mut log: Vec<u16> = vec![];
        let sol = self.solve_rec(
            instance,
            &mut part_sol,
            ret,
            lower,
            upper.makespan,
            timeout,
            &mut log,
        );
        // println!("{:?}", log);
        if sol.is_err() {
            return None;
        }
        let state_length = instance.num_processors + 2;
        analyze_log(log, state_length);

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

fn analyze_log(mut log: Vec<u16>, state_length: usize) {
    let mut current_state = vec![0; state_length];

    let mut state_pointer = 0;

    while state_pointer * state_length < log.len() {
        if log[state_pointer * state_length] != u16::MAX {
            for i in 0..state_length {
                current_state[i] = log[state_pointer * state_length + i];
            }

            let (i, ii, iii) =
                count_nodes_below(&current_state, &mut log, state_pointer, state_length);
            if iii != 0 {
                print!(
                    "({}, {}, {}, {}),",
                    i,
                    ii,
                    iii,
                    current_state[state_length - 1]
                );
            }
        } else {
        }
        state_pointer += 1;
    }
}

fn count_nodes_below(
    current_state: &Vec<u16>,
    log: &mut Vec<u16>,
    state_pointer: usize,
    state_length: usize,
) -> (usize, usize, usize) {
    let level = current_state[state_length - 1];
    let mut state_pointer = state_pointer;
    let mut count = false;
    let mut counted = 0;
    let mut exact_count = 0;
    let mut still_in_first_subtree = true;
    let mut first_subtree_size = 0;

    while (state_pointer + 1) * state_length < log.len() {
        state_pointer += 1;
        if log[state_pointer * state_length + state_length - 1] > level {
            count = false;
            still_in_first_subtree = false;
        } else if log[state_pointer * state_length + state_length - 1] < level {
            if still_in_first_subtree {
                first_subtree_size += 1;
            }
            if count {
                counted += 1;
            }
        } else {
            if &log[state_pointer * state_length..state_pointer * state_length + state_length]
                == current_state
            {
                for i in 0..state_length {
                    log[state_pointer * state_length + i] = u16::MAX;
                }
                count = true;
                counted += 1;
                exact_count += 1;
            } else {
                count = false;
            }
            still_in_first_subtree = false
        }
    }
    return (counted, exact_count, first_subtree_size);
}

#[derive(Clone)]
struct PartialAssignment {
    pub assignment: Vec<usize>,
    pub makespans: Vec<usize>,
    pub unassigned: Vec<usize>,
    pub makespan: usize,
    pub fur_assignments: bitvec::prelude::BitVec,
    pub min_space_required: usize,
    pub time_point_assigned: Vec<usize>,
}
impl PartialAssignment {
    pub fn new(instance: &ProblemInstance) -> PartialAssignment {
        let assignment: Vec<usize> = vec![usize::MAX; instance.num_jobs];
        let makespans: Vec<usize> = vec![0; instance.num_processors];
        let unassigned: Vec<usize> = (0..instance.num_jobs).collect();
        let makespan = 0;
        let fur_assignments: bitvec::prelude::BitVec = bitvec![0;instance.num_jobs];
        let min_space_required: usize = instance.job_sizes.iter().sum();
        let time_point_assigned: Vec<usize> = vec![0; instance.num_jobs];
        return PartialAssignment {
            assignment,
            makespans,
            unassigned,
            makespan,
            fur_assignments,
            min_space_required,
            time_point_assigned,
        };
    }

    pub fn assign(
        &mut self,
        job: usize,
        proc: usize,
        instance: &ProblemInstance,
        fur_assignment: bool,
    ) {
        assert!(self.assignment[job] == usize::MAX);
        self.assignment[job] = proc;
        self.time_point_assigned[job] = self.makespans[proc];
        self.makespans[proc] += instance.job_sizes[job];
        self.makespan = self.makespan.max(self.makespans[proc]);
        let job_pos = self.unassigned.index_of(&job).unwrap();
        if fur_assignment {
            self.fur_assignments.set(job, true);
        }
        self.unassigned.remove(job_pos);
        self.min_space_required -= instance.job_sizes[job];
    }

    pub fn unassign(&mut self, job: usize, instance: &ProblemInstance, fur_assignment: bool) {
        assert!(self.assignment[job] != usize::MAX);
        let proc = self.assignment[job];
        self.time_point_assigned[job] = 0;
        self.assignment[job] = usize::MAX;
        let old_makespan = self.makespans[proc];
        self.makespans[proc] -= instance.job_sizes[job];

        if old_makespan == self.makespan {
            self.makespan = *self.makespans.iter().max().unwrap();
        }

        if fur_assignment {
            self.fur_assignments.set(job, false);
        }
        let mut index = 0;
        while index < self.unassigned.len() && self.unassigned[index] < job {
            index += 1;
        }
        self.unassigned.insert(index, job);
        self.min_space_required += instance.job_sizes[job];
    }
}
