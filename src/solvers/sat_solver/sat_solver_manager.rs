use std::{collections::HashMap, time::Instant};

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

pub struct SatSolverManager {
    pub sat_solver: Box<dyn SatSolver>,
    pub makespan_scheduler: Box<dyn MakespanScheduler>,
    pub encoder: Box<dyn Encoder>,
    pub stats: HashMap<String, f64>,
}

impl SatSolverManager {
    pub fn new(
        sat_solver: Box<dyn SatSolver>,
        makespan_scheduler: Box<dyn MakespanScheduler>,
        encoder: Box<dyn Encoder>,
    ) -> SatSolverManager {
        return SatSolverManager {
            makespan_scheduler,
            sat_solver,
            encoder,
            stats: HashMap::new(),
        };
    }

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
        self.stats = HashMap::new();

        let encoding_time_key: String = "encoding_time".to_owned();
        let num_sat_calls_key: String = "num_sat_calls".to_owned();
        let num_unsat_calls_key: String = "num_unsat_calls".to_owned();
        self.stats.insert(encoding_time_key.clone(), 0.0);
        self.stats.insert(num_sat_calls_key.clone(), 0.0);
        self.stats.insert(num_unsat_calls_key.clone(), 0.0);

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

            let encoding_time = Instant::now();
            let success = self.encoder.basic_encode(
                &partial_solution,
                makespan_to_test,
                timeout,
                500_000_000,
            );
            if !success {
                return None;
            }
            let clauses = self.encoder.output();
            self.stats.insert(
                encoding_time_key.clone(),
                self.stats.get(&encoding_time_key).unwrap() + encoding_time.elapsed().as_secs_f64(),
            );

            let result =
                self.sat_solver
                    .as_mut()
                    .solve(clauses, self.encoder.get_num_vars(), timeout);

            // TODO: there is definetly a better way of doing this, such that we get an any time algo, but i cba rn
            if result.is_timeout() {
                return None;
            }

            let solver_stats = self.sat_solver.get_stats();

            for stat in solver_stats {
                if self.stats.contains_key(&stat.0) {
                    self.stats
                        .insert(stat.0.clone(), self.stats.get(&stat.0).unwrap() + stat.1);
                } else {
                    self.stats.insert(stat.0.clone(), stat.1);
                }
            }

            let var_assingment = result.unwrap();

            if var_assingment.is_none() {
                lower = makespan_to_test + 1;

                self.stats.insert(
                    num_unsat_calls_key.clone(),
                    self.stats.get(&num_unsat_calls_key).unwrap() + 1.0,
                );

                if verbose {
                    println!("UNSAT");
                }
            } else {
                let mss: MSS = MSS::new();
                let old_bound = solution.makespan;
                solution = self
                    .encoder
                    .decode(instance, var_assingment.as_ref().unwrap());
                // TODO: put this back in after verfyfing that everything works
                let (_, improved_solution) =
                    mss.bound(instance, lower, Some(solution), &Timeout::new(1.0));
                solution = improved_solution.unwrap();
                let new_bound = solution.makespan;

                self.stats.insert(
                    num_sat_calls_key.clone(),
                    self.stats.get(&num_sat_calls_key).unwrap() + 1.0,
                );

                if verbose {
                    println!("SAT");
                }
                if old_bound <= new_bound {
                    println!("what, how did the solution get worse, was {}, but now is\n{} on an instance n{} m{}", old_bound ,solution, partial_solution.instance.num_jobs, partial_solution.instance.num_processors);
                    //println!("{:?}", solution.assignment.iter().enumerate().filter(|(i,x)| **x == 74).map(|(i,x)| partial_solution.instance.job_sizes[i]).collect::<Vec<usize>>());
                    break;
                }
            }

            if verbose {
                println!("lower: {} upper {}", lower, solution.makespan);
            }
        }

        return Some(solution);
    }
}
