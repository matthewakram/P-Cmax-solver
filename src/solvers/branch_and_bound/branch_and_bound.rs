use std::collections::{HashMap, HashSet};

use bitvec::bitvec;

use crate::{
    bdd::bdd_dyn::RangeTable,
    common::common::IndexOf,
    problem_instance::{
        partial_solution::{PartialSolution, self}, problem_instance::ProblemInstance, solution::Solution,
    },
    problem_simplification::{
        fill_up_rule::FillUpRule, final_simp_rule::FinalizeRule, half_size_rule::HalfSizeRule,
        simplification_rule::SimpRule,
    },
    solvers::solver_manager::SolverManager,
};

#[derive(Clone)]
pub struct BranchAndBound {}

impl BranchAndBound {
    pub fn new() -> BranchAndBound {
        return BranchAndBound {};
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

impl BranchAndBound {
    fn solve_rec(
        &self,
        instance: &ProblemInstance,
        // we maintain the for part_sol.makespan < upper
        part_sol: &mut PartialAssignment,
        ret: &mut RangeTable,
        lower: usize,
        best_makespan_found: usize,
        timeout: &crate::common::timeout::Timeout,
    ) -> Result<Option<Solution>, ()> {
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
            first_part_sol.assign(first_part_sol.unassigned[0], second_min_proc, instance, false);
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
                let next_makespan_to_check = better_option.makespan - 1;
                *ret = RangeTable::new(
                    &(0..instance.num_jobs).into_iter().collect(),
                    &instance.job_sizes,
                    next_makespan_to_check,
                );
                println!("solution found with makespan {}", better_option.makespan);
                return Ok(Some(Solution {
                    assignment: better_option.assignment,
                    makespan: better_option.makespan,
                }));
            } else {
                return Ok(None);
            }
        }

        let mut fur_job = usize::MAX;
        let mut fur_proc = usize::MAX;
        for proc in 0..instance.num_processors {
            let proc_makespan = part_sol.makespans[proc];
            if best_makespan_found - proc_makespan
                < instance.job_sizes[*part_sol.unassigned.last().unwrap()]
            {
                continue;
            }
            for job in &part_sol.unassigned {
                if best_makespan_found - proc_makespan > instance.job_sizes[*job] {
                    if ret.get_range(*job, best_makespan_found - 1 - instance.job_sizes[*job])
                        == ret.get_range(*job, proc_makespan)
                    {
                        fur_job = *job;
                        fur_proc = proc;
                    }
                    break;
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

                // now we have to revert the FUR decision, and recurse
                let better_sol =
                    self.solve_rec(instance, part_sol, ret, lower, best_makespan_found, timeout);
                if better_sol.is_err() {
                    return Err(());
                }
                let better_sol = better_sol.unwrap();
                if better_sol.is_none() {
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
        let mut best_sol: Option<Solution> = None;
        let mut best_makespan_found = best_makespan_found;

        let job_to_branch_on_size = instance.job_sizes[job_to_branch_on];
        let mut precedence = usize::MAX;
        for i in (0..job_to_branch_on).rev() {
            if instance.job_sizes[i] != job_to_branch_on_size {
                break;
            } else if !part_sol.fur_assignments[i] {
                precedence = part_sol.assignment[i];
                break;
            }
        }
        let precedence = if precedence == usize::MAX {0} else {precedence};
        let mut procs: Vec<(usize, usize)> = part_sol
            .makespans
            .iter()
            .enumerate()
            .filter(|(proc_num, makespan)| {
                best_makespan_found - **makespan > instance.job_sizes[job_to_branch_on]
                    && proc_num >= &precedence
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
            let range = ret
                .get_range(job_to_branch_on, part_sol.makespans[proc])
                .unwrap();
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
            }
        }

        return Ok(best_sol);
    }
}

impl SolverManager for BranchAndBound {
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

        let ret = &mut RangeTable::new(
            &(0..instance.num_jobs).into_iter().collect(),
            &instance.job_sizes,
            makespan_to_test,
        );

        let sol = self.solve_rec(instance, &mut part_sol, ret, lower, upper.makespan, timeout);
        if sol.is_err() {
            return None;
        }
        let sol = sol.unwrap();
        if sol.is_none() {
            return Some(upper.clone());
        } else {
            return sol;
        }
    }
}

#[derive(Clone)]
struct PartialAssignment {
    pub assignment: Vec<usize>,
    pub makespans: Vec<usize>,
    pub unassigned: Vec<usize>,
    pub makespan: usize,
    pub fur_assignments: bitvec::prelude::BitVec,
    pub min_space_required: usize,
}
impl PartialAssignment {
    pub fn new(instance: &ProblemInstance) -> PartialAssignment {
        let assignment: Vec<usize> = vec![usize::MAX; instance.num_jobs];
        let makespans: Vec<usize> = vec![0; instance.num_processors];
        let unassigned: Vec<usize> = (0..instance.num_jobs).collect();
        let makespan = 0;
        let fur_assignments: bitvec::prelude::BitVec = bitvec![0;instance.num_jobs];
        let min_space_required: usize = instance.job_sizes.iter().sum();
        return PartialAssignment {
            assignment,
            makespans,
            unassigned,
            makespan,
            fur_assignments,
            min_space_required,
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
        self.makespans[proc] += instance.job_sizes[job];
        self.makespan = self.makespan.max(self.makespans[proc]);
        let job_pos = self.unassigned.index_of(&job).unwrap();
        if fur_assignment {
            self.fur_assignments.set(job, true);
        }
        self.unassigned.remove(job_pos);
        self.min_space_required -= instance.job_sizes[job];
    }

    pub fn unassign(
        &mut self,
        job: usize,
        instance: &ProblemInstance,
        fur_assignment: bool,
    ) {
        assert!(self.assignment[job] != usize::MAX);
        let proc = self.assignment[job];
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
