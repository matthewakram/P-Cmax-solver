use std::collections::HashMap;

use bitvec::bitvec;

use crate::{
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

use super::{ret::RET, ussl::USSL};

#[derive(Clone)]
pub struct CDSM {
    stats: HashMap<String, f64>,
    last_state_at_level: Vec<Vec<u32>>,
    inter_rule: bool,
    fur_rule: bool,
    irrelevance_rule: bool,
    last_size_rule: bool,
    state_mem: bool,
    mem_limit: usize,
}

impl CDSM {
    pub fn new() -> CDSM {
        return CDSM {
            stats: HashMap::new(),
            last_state_at_level: vec![],
            inter_rule: true,
            fur_rule: true,
            irrelevance_rule: true,
            last_size_rule: true,
            state_mem: true,
            mem_limit: usize::MAX,
        };
    }

    pub fn new_with_rules(
        inter_rule: bool,
        fur_rule: bool,
        irrelevance_rule: bool,
        last_size_rule: bool,
        state_mem: bool,
        mem_limit: usize,
    ) -> CDSM {
        return CDSM {
            stats: HashMap::new(),
            last_state_at_level: vec![],
            inter_rule,
            fur_rule,
            irrelevance_rule,
            last_size_rule,
            state_mem,
            mem_limit,
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

impl CDSM {
    fn gen_state_list(
        &mut self,
        part_sol: &PartialAssignment,
        ret: &RET,
        instance: &ProblemInstance,
    ) -> &Vec<u32> {
        assert!(self.state_mem);
        let largest_unassigned = part_sol.unassigned[0];
        for i in 0..instance.num_processors {
            self.last_state_at_level[largest_unassigned][i] = part_sol.makespan_sans_fur[i] as u32;
        }
        for fur_job in 0..largest_unassigned {
            if part_sol.fur_assignments[fur_job] {
                self.last_state_at_level[largest_unassigned][part_sol.assignment[fur_job]] +=
                    instance.job_sizes[fur_job] as u32;
            }
        }
        for i in 0..instance.num_processors {
            self.last_state_at_level[largest_unassigned][i] = ret.get_range(
                largest_unassigned,
                self.last_state_at_level[largest_unassigned][i] as usize,
            ) as u32;
        }
        self.last_state_at_level[largest_unassigned][instance.num_processors] = u32::MAX;
        self.last_state_at_level[largest_unassigned].sort();
        self.last_state_at_level[largest_unassigned][instance.num_processors] =
            largest_unassigned as u32;
        // println!("{:?}", list);
        return &self.last_state_at_level[largest_unassigned];
    }

    fn get_state_list(&self, part_sol: &PartialAssignment) -> &Vec<u32> {
        assert!(self.state_mem);
        let largest_unassigned = part_sol.unassigned[0];
        return &self.last_state_at_level[largest_unassigned];
    }

    fn solve_rec(
        &mut self,
        instance: &ProblemInstance,
        // we maintain the for part_sol.makespan < best_makespan_found
        part_sol: &mut PartialAssignment,
        ret: &mut RET,
        saved_states: &mut USSL,
        lower: usize,
        best_makespan_found: usize,
        timeout: &crate::common::timeout::Timeout,
    ) -> Result<(Option<PartialAssignment>, usize), ()> {
        if timeout.time_finished() {
            return Result::Err(());
        }
        part_sol.num_nodes_explored += 1;
        let lower: usize = lower.max(part_sol.makespan);
        if best_makespan_found <= lower {
            return Ok((None, 0));
        }
        assert!(part_sol.makespan < best_makespan_found);

        let remaining_space: usize = part_sol
            .makespans
            .iter()
            .map(|x| best_makespan_found - 1 - *x)
            .filter(|x| x >= &instance.job_sizes[*part_sol.unassigned.last().unwrap()])
            .sum();

        if part_sol.min_space_required > remaining_space {
            return Ok((None, 0));
        }
        // ====================================== END CONDITIONS

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
                if (self.fur_rule || self.inter_rule || self.state_mem)
                    && next_makespan_to_check != instance.job_sizes[0]
                {
                    let last_relevant_index =
                        self.calculate_irrelevance_index(instance, next_makespan_to_check);
                    ret.decrease_makespan_to(
                        &instance.job_sizes,
                        next_makespan_to_check,
                        last_relevant_index,
                    );
                }
                if self.state_mem {
                    saved_states.clear_all();
                }
                return Ok((Some(better_option), 0));
            } else {
                return Ok((None, 0));
            }
        }

        if self.last_size_rule {
            // TODO: line below here, only consider relevant jobs
            if instance.job_sizes[part_sol.unassigned[0]]
                == instance.job_sizes[part_sol.unassigned[part_sol.unassigned.len() - 1]]
            {
                let mut sol = part_sol.clone();

                for &unassigned_job in &part_sol.unassigned {
                    let mut min_proc = 0;

                    for proc in 0..instance.num_processors {
                        if sol.makespans[proc] < sol.makespans[min_proc] {
                            min_proc = proc;
                        }
                    }

                    sol.assign(unassigned_job, min_proc, instance, false);

                    if sol.makespan >= best_makespan_found {
                        return Ok((None, 0));
                    }
                }

                let next_makespan_to_check = sol.makespan - 1;
                if self.fur_rule || self.inter_rule || self.state_mem {
                    if next_makespan_to_check != instance.job_sizes[0] {
                        let last_relevant_index =
                            self.calculate_irrelevance_index(instance, next_makespan_to_check);
                        ret.decrease_makespan_to(
                            &instance.job_sizes,
                            next_makespan_to_check,
                            last_relevant_index,
                        );
                    }
                    if self.state_mem {
                        saved_states.clear_all();
                    }
                }
                return Ok((Some(sol), 0));
            }
        }

        if self.irrelevance_rule {
            if !ret.is_relevant(part_sol.unassigned[0]) {
                println!(
                    "ret irrelevance index is: {} and num jobs is {} job in question {}",
                    self.calculate_irrelevance_index(instance, best_makespan_found),
                    instance.num_jobs,
                    part_sol.unassigned[0]
                );
                let mut sol = part_sol.clone();

                for &unassigned_job in &part_sol.unassigned {
                    let mut min_proc = 0;

                    for proc in 0..instance.num_processors {
                        if sol.makespans[proc] < sol.makespans[min_proc] {
                            min_proc = proc;
                        }
                    }

                    sol.assign(unassigned_job, min_proc, instance, false);

                    assert!(sol.makespan < best_makespan_found);
                }

                let next_makespan_to_check = sol.makespan - 1;
                if next_makespan_to_check != instance.job_sizes[0] {
                    let last_relevant_index =
                        self.calculate_irrelevance_index(instance, next_makespan_to_check);
                    ret.decrease_makespan_to(
                        &instance.job_sizes,
                        next_makespan_to_check,
                        last_relevant_index,
                    );
                }
                if self.state_mem {
                    saved_states.clear_all();
                }

                let better_sol = self.solve_rec(
                    instance,
                    part_sol,
                    ret,
                    saved_states,
                    lower,
                    sol.makespan,
                    timeout,
                );

                if better_sol.is_err() {
                    return Err(());
                }

                let (better_sol, num_nodes) = better_sol.unwrap();
                if better_sol.is_none() {
                    return Ok((Some(sol), 0));
                } else {
                    return Ok((better_sol, num_nodes));
                }
            }
        }

        // ========================================================

        if self.state_mem {
            let original_state = self.gen_state_list(part_sol, ret, &instance);
            if saved_states.is_present(original_state) {
                return Ok((None, 0));
            }
        }

        if self.fur_rule {
            let mut fur_job: usize = usize::MAX;
            let mut fur_proc = usize::MAX;
            for proc in 0..instance.num_processors {
                let proc_makespan = part_sol.makespans[proc];
                if best_makespan_found - proc_makespan
                    < instance.job_sizes[*part_sol.unassigned.last().unwrap()]
                {
                    continue;
                }
                for i in 0..part_sol.unassigned.len() {
                    let job = part_sol.unassigned[i];
                    if !ret.is_relevant(job) {
                        break;
                    }
                    if best_makespan_found - proc_makespan > instance.job_sizes[job] {
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

            if fur_job != usize::MAX {
                // in this case we use the FUR and recurse

                part_sol.assign(fur_job, fur_proc, instance, true);
                let sol = self.solve_rec(
                    instance,
                    part_sol,
                    ret,
                    saved_states,
                    lower,
                    best_makespan_found,
                    timeout,
                );
                if sol.is_err() {
                    return Err(());
                }
                part_sol.unassign(fur_job, instance, true);
                let (sol, num_branches) = sol.unwrap();
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
                        return Ok((Some(sol), num_branches + 1));
                    }

                    // now we have to revert the FUR decision, and recurse
                    let better_sol = self.solve_rec(
                        instance,
                        part_sol,
                        ret,
                        saved_states,
                        lower,
                        best_makespan_found,
                        timeout,
                    );
                    if better_sol.is_err() {
                        return Err(());
                    }
                    let (better_sol, num_sub_branches) = better_sol.unwrap();

                    if better_sol.is_none() {
                        //println!("happened");
                        return Ok((Some(sol), num_branches + num_sub_branches + 1));
                    } else {
                        let sol = better_sol.unwrap();
                        return Ok((Some(sol), num_branches + num_sub_branches + 1));
                    }
                } else {
                    // here sol is None, so we know we cannot improve upon best_makespan_found
                    return Ok((None, num_branches + 1));
                }
            }
        }

        // if we reach this point, we know we cannot use the FUR rule here. Thus our only option is to branch
        let job_to_branch_on = part_sol.unassigned[0];

        // In order to direct the search towards solutions more quickly, we sort the procs in decreasing order of available makespan
        let mut best_sol: Option<PartialAssignment> = None;
        let mut best_makespan_found = best_makespan_found;

        let job_to_branch_on_size = instance.job_sizes[job_to_branch_on];
        let mut precedence = 0;

        if job_to_branch_on > 0 && instance.job_sizes[job_to_branch_on - 1] == job_to_branch_on_size
        {
            for prev_job in (0..job_to_branch_on).rev() {
                if instance.job_sizes[prev_job] != job_to_branch_on_size {
                    break;
                }
                if !part_sol.fur_assignments[prev_job] {
                    precedence = part_sol.time_point_assigned[prev_job];
                    break;
                }
            }
        }

        let mut last_range: usize = usize::MAX;
        let mut last_tried_makespan = usize::MAX;
        let mut num_branches: usize = 0;
        loop {
            let mut next_proc = usize::MAX;
            let mut next_proc_makespan = usize::MAX;
            for proc in 0..instance.num_processors {
                if part_sol.makespans[proc] >= precedence
                    && part_sol.makespans[proc] < next_proc_makespan
                    && (last_tried_makespan == usize::MAX
                        || part_sol.makespans[proc] > last_tried_makespan)
                {
                    next_proc = proc;
                    next_proc_makespan = part_sol.makespans[proc];
                }
            }
            last_tried_makespan = next_proc_makespan;

            if next_proc_makespan == usize::MAX
                || best_makespan_found <= instance.job_sizes[job_to_branch_on] + next_proc_makespan
            {
                break;
            }

            let range = if self.inter_rule {
                ret.get_range(job_to_branch_on, part_sol.makespans[next_proc]) as usize
            } else {
                next_proc_makespan
            };

            if last_range == range {
                continue;
            }
            last_range = range;

            part_sol.assign(job_to_branch_on, next_proc, instance, false);
            let sol = self.solve_rec(
                instance,
                part_sol,
                ret,
                saved_states,
                lower,
                best_makespan_found,
                timeout,
            );

            if sol.is_err() {
                return Err(());
            }
            part_sol.unassign(job_to_branch_on, instance, false);

            let (sol, num_sub_branches) = sol.unwrap();
            if sol.is_some() {
                let sol = sol.unwrap();
                num_branches = num_sub_branches;
                best_makespan_found = sol.makespan;
                best_sol = Some(sol);
                if best_makespan_found <= lower {
                    if self.state_mem && part_sol.makespan < best_makespan_found {
                        saved_states.insert_list(self.gen_state_list(part_sol, ret, &instance));
                    }
                    return Ok((best_sol, num_branches + 1));
                }
                if self.inter_rule {
                    last_range = ret.get_range(job_to_branch_on, next_proc_makespan) as usize;
                }
            } else {
                num_branches += num_sub_branches;
            }
        }

        if self.state_mem && best_sol.is_none() {
            saved_states.insert_list(self.get_state_list(part_sol));
        }

        return Ok((best_sol, num_branches + 1));
    }

    ///*
    /// Returns the last relvant job index. all indices after this can be ignored when solving the PCmax decision problem
    ///  */
    fn calculate_irrelevance_index(
        &self,
        instance: &ProblemInstance,
        makespan_to_test: usize,
    ) -> usize {
        if !self.irrelevance_rule {
            return instance.num_jobs - 1;
        }
        let mut sum_of_prev_jobs: usize = instance.job_sizes.iter().sum();
        for job in (0..instance.num_jobs).rev() {
            sum_of_prev_jobs -= instance.job_sizes[job];
            if sum_of_prev_jobs
                >= instance.num_processors * (makespan_to_test - instance.job_sizes[job] + 1)
            {
                return job;
            }
        }
        return 0;
    }
}

impl SolverManager for CDSM {
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

        let last_relevant_index = self.calculate_irrelevance_index(instance, makespan_to_test);
        let mut ret = if self.fur_rule || self.inter_rule || self.state_mem {
            RET::new(
                &instance.job_sizes,
                makespan_to_test,
                last_relevant_index,
                self.mem_limit,
            )
        } else {
            RET::new(&vec![], 1, 0, self.mem_limit)
        };

        let mut saved_states = USSL::new(instance.num_processors + 1, 1000, 3, 10, self.mem_limit);
        self.last_state_at_level = vec![vec![0; instance.num_processors + 1]; instance.num_jobs];

        let sol = self.solve_rec(
            instance,
            &mut part_sol,
            &mut ret,
            &mut saved_states,
            lower,
            upper.makespan,
            timeout,
        );
        //println!("{:?}", saved_states);
        // println!("num_nodes_explored {}", part_sol.num_nodes_explored);
        self.stats.insert(
            "num_nodes_explored".to_string(),
            part_sol.num_nodes_explored as f64,
        );
        self.stats.insert(
            "mem_used".to_owned(),
            ((ret.get_space_consuption()
                + saved_states.mem_usage()) as f64));
        if sol.is_err() {
            return None;
        }
        let (sol, _) = sol.unwrap();
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
    pub makespan_sans_fur: Vec<usize>,
    pub fur_assignments: bitvec::prelude::BitVec,
    pub min_space_required: usize,
    pub time_point_assigned: Vec<usize>,
    pub num_nodes_explored: usize,
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
        let makespan_sans_fur: Vec<usize> = vec![0; instance.num_processors];
        return PartialAssignment {
            assignment,
            makespans,
            unassigned,
            makespan,
            fur_assignments,
            min_space_required,
            time_point_assigned,
            makespan_sans_fur,
            num_nodes_explored: 0,
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

        if !fur_assignment {
            self.makespan_sans_fur[proc] += instance.job_sizes[job];
        }
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

        if !fur_assignment {
            self.makespan_sans_fur[proc] -= instance.job_sizes[job];
        }

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
