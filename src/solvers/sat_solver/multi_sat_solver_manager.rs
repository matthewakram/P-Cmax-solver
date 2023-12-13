use std::{
    process::Command,
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    bounds::{bound::Bound, upper_bounds::mss::MSS},
    common::timeout::Timeout,
    encoding::encoder::Encoder,
    makespan_scheduling::makespan_scheduler::MakespanScheduler,
    problem_instance::{
        partial_solution::PartialSolution, problem_instance::ProblemInstance, solution::Solution,
    },
    problem_simplification::{
        fill_up_rule::FillUpRule, final_simp_rule::FinalizeRule, half_size_rule::HalfSizeRule,
        simplification_rule::SimpRule,
    },
    solvers::solver::SatSolver,
};

pub struct MultiSatSolverManager {
    pub sat_solver: Box<dyn SatSolver>,
    pub unsat_solver: Box<dyn SatSolver>,
    pub makespan_scheduler: Box<dyn MakespanScheduler>,
    pub sat_encoder: Box<dyn Encoder>,
    pub unsat_encoder: Box<dyn Encoder>,
}

impl MultiSatSolverManager {
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

            // ----------- SAT Preparation -------------------------

            let mut sat_encoder = self.sat_encoder.clone();
            let sat_finish = timeout.get_finish_var();
            let sat_timeout = timeout.clone();
            let sat_partial_solution_clone = partial_solution.clone();
            let mut sat_solver = self.sat_solver.clone();
            let sat_pid: Arc<Mutex<usize>> = sat_solver.get_pid();

            // ----------- UNSAT Preparation -------------------------

            let mut unsat_encoder = self.unsat_encoder.clone();
            let unsat_finish = timeout.get_finish_var();
            let unsat_timeout = timeout.clone();
            let unsat_partial_solution_clone = partial_solution.clone();
            let mut unsat_solver = self.unsat_solver.clone();
            let unsat_pid: Arc<Mutex<usize>> = unsat_solver.get_pid();

            // ----------- SAT QUERY-------------------------

            let sat_query: thread::JoinHandle<(bool, Option<Solution>)> =
                thread::spawn(move || {
                    let success = sat_encoder.basic_encode(
                        &sat_partial_solution_clone,
                        makespan_to_test,
                        &sat_timeout,
                        500_000_000,
                    );
                    if success == false {
                        return (true, None);
                    }
                    //thread::sleep(Duration::from_secs_f64(10.0));
                    let formula = sat_encoder.output();
                    let res = sat_solver.solve(formula, sat_encoder.get_num_vars(), &sat_timeout);

                    sat_finish.store(true, std::sync::atomic::Ordering::Relaxed);
                    let mut unsat_pid = unsat_pid.lock().unwrap();
                    if *unsat_pid != 0 {
                        // TODO: Correct this for windows!
                        let _ = Command::new("kill")
                            .arg(unsat_pid.to_string())
                            .output()
                            .unwrap();
                        *unsat_pid = 0;
                    }
                    if res.is_timeout() {
                        return (true, None);
                    }
                    if res.is_unsat() {
                        return (false, None);
                    }
                    if res.is_sat() {
                        let res = sat_encoder
                            .decode(&sat_partial_solution_clone.instance, &res.unwrap().unwrap());
                        let mss: MSS = MSS::new();
                        let (_, improved_solution) = mss.bound(
                            &sat_partial_solution_clone.instance,
                            lower,
                            Some(res),
                            &Timeout::new(2.0),
                        );
                        let res = improved_solution.unwrap();
                        return (false, Some(res));
                    }

                    panic!("none of the above???");
                });

            // ----------- UNSAT QUERY-------------------------
            let unsat_query: thread::JoinHandle<(bool, Option<Solution>)> =
                thread::spawn(move || {
                    let success = unsat_encoder.basic_encode(
                        &unsat_partial_solution_clone,
                        makespan_to_test,
                        &unsat_timeout,
                        500_000_000,
                    );
                    if success == false {
                        return (true, None);
                    }
                    let formula = unsat_encoder.output();
                    let res =
                        unsat_solver.solve(formula, unsat_encoder.get_num_vars(), &unsat_timeout);

                    unsat_finish.store(true, std::sync::atomic::Ordering::Relaxed);
                    let mut sat_pid = sat_pid.lock().unwrap();
                    if *sat_pid != 0 {
                        // TODO: Correct this for windows!
                        let _ = Command::new("kill")
                            .arg(sat_pid.to_string())
                            .output()
                            .unwrap();
                        *sat_pid = 0;
                    }

                    if res.is_timeout() {
                        return (true, None);
                    }
                    if res.is_unsat() {
                        return (false, None);
                    }
                    if res.is_sat() {
                        let res = unsat_encoder.decode(
                            &unsat_partial_solution_clone.instance,
                            &res.unwrap().unwrap(),
                        );
                        let mss: MSS = MSS::new();
                        let (_, improved_solution) = mss.bound(
                            &unsat_partial_solution_clone.instance,
                            lower,
                            Some(res),
                            &Timeout::new(2.0),
                        );
                        let res = improved_solution.unwrap();
                        return (false, Some(res));
                    }

                    panic!("none of the above???");
                });

            // -----------------------------------------------

            let (sat_timeout, sat_result) = sat_query.join().unwrap();
            let (unsat_timeout, unsat_result) = unsat_query.join().unwrap();
            timeout.reset_finish();

            if sat_timeout && unsat_timeout {
                return None;
            }
            if !sat_timeout {
                if !(unsat_timeout
                    || (unsat_result.is_none() && sat_result.is_none())
                    || (unsat_result.is_some() && sat_result.is_some()))
                {
                    println!(
                        " the values are {} {} {} {} {}, ",
                        unsat_timeout,
                        unsat_result.is_none(),
                        sat_result.is_none(),
                        unsat_result.is_some(),
                        sat_result.is_some()
                    );
                    println!("the error occured in {:?}", instance);
                }
                assert!(
                    unsat_timeout
                        || (unsat_result.is_none() && sat_result.is_none())
                        || (unsat_result.is_some() && sat_result.is_some())
                );

                if sat_result.is_none() {
                    // the result is that this is UNSAT
                    lower = makespan_to_test + 1;
                    if verbose {
                        println!("sat: UNSAT");
                    }
                } else {
                    solution = sat_result.unwrap();
                    if verbose {
                        println!("sat: SAT");
                    }
                }
            } else {
                if unsat_result.is_none() {
                    // the result is that this is UNSAT
                    lower = makespan_to_test + 1;
                    if verbose {
                        println!("unsat: UNSAT");
                    }
                } else {
                    solution = unsat_result.unwrap();
                    if verbose {
                        println!("unsat: SAT");
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
