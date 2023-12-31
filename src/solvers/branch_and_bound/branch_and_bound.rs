use std::collections::HashSet;


use crate::{
    bdd::bdd_dyn::RangeTable,
    common::common::IndexOf,
    problem_instance::{
        partial_solution::PartialSolution,
        problem_instance::ProblemInstance,
        solution::Solution,
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

fn min_proc(part_sol: &PartialAssignment) -> usize {
    let mut min = usize::MAX;
    let mut min_proc = 0;
    for i in 0..part_sol.makespans.len() {
        if part_sol.makespans[i] < min {
            min = part_sol.makespans[i];
            min_proc = i;
        }
    }
    return min_proc;
}

impl BranchAndBound {
    fn solve_rec(
        &self,
        instance: &ProblemInstance,
        // we maintain the for part_sol.makespan < upper
        mut part_sol: PartialAssignment,
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
        let min_space_needed: usize = part_sol
            .unassigned
            .iter()
            .map(|x| instance.job_sizes[*x])
            .sum();
        if min_space_needed > remaining_space {
            return Ok(None);
        }
        // ======================================

        // TODO do this for unassigned.len() == 2
        if part_sol.unassigned.len() == 1 {
            let min_proc = min_proc(&part_sol);
            if part_sol.makespans[min_proc] + instance.job_sizes[part_sol.unassigned[0]]
                < best_makespan_found
            {
                part_sol.assign(part_sol.unassigned[0], min_proc, instance);
                assert!(best_makespan_found > part_sol.makespan);
                let next_makespan_to_check = part_sol.makespan - 1;
                *ret = RangeTable::new(
                    &(0..instance.num_jobs).into_iter().collect(),
                    &instance.job_sizes,
                    next_makespan_to_check,
                );
                println!("solution found with makespan {}", part_sol.makespan);
                return Ok(Some(Solution {
                    assignment: part_sol.assignment,
                    makespan: part_sol.makespan,
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
            let mut part_sol_prime = part_sol.clone();
            part_sol_prime.assign(fur_job, fur_proc, instance);
            let sol = self.solve_rec(
                instance,
                part_sol_prime.clone(),
                ret,
                lower,
                best_makespan_found,
                timeout,
            );
            if sol.is_err() {
                return Err(());
            }
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

        // In order to direct the search towards solutions more quickly, we roughly sort the procs in decreasing order of available makespan
        let mut best_sol: Option<Solution> = None;
        let mut best_makespan_found = best_makespan_found;

        let mut procs: Vec<(usize, usize)> = part_sol
            .makespans
            .iter()
            .filter(|makespan| {
                best_makespan_found - **makespan > instance.job_sizes[job_to_branch_on]
            })
            .enumerate()
            .map(|(x, a)| (x, *a))
            .collect();
        procs.sort_by(|(_, makespan1), (_, makespan2)| makespan1.cmp(makespan2));
        let procs_to_branch_on: Vec<usize> = procs.iter().map(|(proc, _)| *proc).collect();
        let mut seen_ranges: HashSet<usize> = HashSet::new();

        for proc in procs_to_branch_on {
            if best_makespan_found - part_sol.makespans[proc]
                <= instance.job_sizes[job_to_branch_on]
            {
                continue;
            }
            let range = ret
                .get_range(job_to_branch_on, part_sol.makespans[proc])
                .unwrap();
            if seen_ranges.contains(&range) {
                continue;
            }
            seen_ranges.insert(range);

            let mut part_sol_prime = part_sol.clone();
            part_sol_prime.assign(job_to_branch_on, proc, instance);
            let sol = self.solve_rec(
                instance,
                part_sol_prime,
                ret,
                lower,
                best_makespan_found,
                timeout,
            );

            if sol.is_err() {
                return Err(());
            }

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
        let part_sol = PartialAssignment::new(instance);

        let ret = &mut RangeTable::new(
            &(0..instance.num_jobs).into_iter().collect(),
            &instance.job_sizes,
            makespan_to_test,
        );

        let sol = self.solve_rec(instance, part_sol, ret, lower, upper.makespan, timeout);
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
}
impl PartialAssignment {
    pub fn new(instance: &ProblemInstance) -> PartialAssignment {
        let assignment: Vec<usize> = vec![usize::MAX; instance.num_jobs];
        let makespans: Vec<usize> = vec![0; instance.num_processors];
        let unassigned: Vec<usize> = (0..instance.num_jobs).collect();
        let makespan = 0;

        return PartialAssignment {
            assignment,
            makespans,
            unassigned,
            makespan,
        };
    }

    pub fn assign(&mut self, job: usize, proc: usize, instance: &ProblemInstance) {
        assert!(self.assignment[job] == usize::MAX);
        self.assignment[job] = proc;
        self.makespans[proc] += instance.job_sizes[job];
        self.makespan = self.makespan.max(self.makespans[proc]);
        let job_pos = self.unassigned.index_of(&job).unwrap();
        self.unassigned.remove(job_pos);
    }
}
