use crate::{
    encoding::encoder::Encoder,
    makespan_scheduling::makespan_scheduler::MakespanScheduler,
    problem_instance::{
        partial_solution::PartialSolution,
        problem_instance::ProblemInstance,
        solution::{Solution, self},
    },
    problem_simplification::{
        fill_up_rule::FillUpRule, final_simp_rule::FinalizeRule, half_size_rule::HalfSizeRule,
        simplification_rule::SimpRule,
    },
    randomized_checkers::{
        randomized_checker::RandomizedChecker, randomized_multi_sss_randomized_checker::RandomizedMultiSSSRandomizedChecker,
    },
    solvers::solver::SatSolver, common::timeout::Timeout, bounds::{upper_bounds::mss::MSS, bound::Bound},
};

pub struct SatSolverManager {
    pub sat_solver: Box<dyn SatSolver>,
    pub makespan_scheduler: Box<dyn MakespanScheduler>,
    pub encoder: Box<dyn Encoder>,
}

impl SatSolverManager {
    pub fn solve(
        &mut self,
        instance: &ProblemInstance,
        lower: usize,
        upper: &Solution,
        timeout: &Timeout,
        verbose: bool,
    ) -> Option<Solution> {
        let mut solution: Solution = upper.clone();
        let mut lower = lower;

        let mut simp_useful = false;

        while lower != solution.makespan {
            let makespan_to_test = self.makespan_scheduler.next_makespan(
                instance,
                &solution,
                lower,
                solution.makespan,
            );
            if verbose {
                println!("makespan to test: {}", makespan_to_test);
            }


            // Here We refine the solution that we have
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
                lower = makespan_to_test + 1;
                continue;
            }
            let partial_solution = partial_solution.unwrap();
            //println!("job sizes {:?}", partial_solution.instance.job_sizes);
            //println!("assigned {:?}", partial_solution.assigned_makespan);
            //for a in &partial_solution.possible_allocations {
            //    println!("poss all {:?}", a);
            //}

            let mut var_assingment = None;

            

            //if simp_useful {
            //    let random = RandomizedMultiSSSRandomizedChecker {};
            //    if timeout.time_finished() {
            //        return None;
            //    }
            //    
            //    let sol = random.is_sat(&partial_solution, makespan_to_test, &Timeout::new(timeout.remaining_time()/3.0));
            //    if sol.is_some() {
            //        let sol = sol.unwrap();
            //        assert!(sol.makespan <= makespan_to_test);
            //        solution = sol;
            //        //println!("SAT by simp");
            //        continue;
            //    }
            //}
            //simp_useful = false;

            if var_assingment.is_none() {
                self.encoder
                    .basic_encode(&partial_solution, makespan_to_test, timeout);
                let clauses = self.encoder.output();
                //let file_name: &str = "./test";
                //println!("writing to file, formula size {}", clauses.len());
                //input_output::to_dimacs::print_to_dimacs(
                //    file_name,
                //    clauses,
                //    self.encoder.get_num_vars(),
                //);
                //println!("wrote to file");
                let result = self.sat_solver.as_ref().solve(&clauses, self.encoder.get_num_vars(), timeout);

                // TODO: there is definetly a better way of doing this, such that we get an any time algo, but i cba rn
                if result.is_timeout() {
                    return None;
                }

                var_assingment = result.unwrap();

                if var_assingment.is_none() {
                    lower = makespan_to_test + 1;
                    if verbose {
                        println!("UNSAT");
                    }
                } else {
                    let mss: MSS = MSS {};
                    let old_bound = solution.makespan;
                    solution = self
                        .encoder
                        .decode(instance, var_assingment.as_ref().unwrap());
                    let (_, improved_solution) = mss.bound(instance, lower, Some(solution), &Timeout::new(1.0));
                    solution = improved_solution.unwrap();
                    let new_bound = solution.makespan;
                    if verbose {
                        println!("SAT");
                    }
                    if old_bound <= new_bound {
                        println!("what, how did the solution get worse {}", solution);
                        break;
                    }
                }
            }

            if verbose {
                println!("lower: {} upper {}", lower, solution.makespan);
            }
        }

        return Some(solution);
    }
}
