use crate::{
    encoding::encoder::Encoder,
    input_output,
    makespan_scheduling::makespan_scheduler::MakespanScheduler,
    problem_instance::{
        partial_solution::PartialSolution,
        problem_instance::ProblemInstance,
        solution::Solution,
    },
    problem_simplification::{
        fill_up_rule::FillUpRule, half_size_rule::HalfSizeRule, simplification_rule::SimpRule, final_simp_rule::FinalizeRule,
    },
    solvers::solver::SatSolver,
};
use std::time::Instant;

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
        timeout: f64,
        verbose: bool
    ) -> Option<Solution> {
        let mut solution: Solution = upper.clone();
        let mut lower = lower;

        let mut timeout = timeout;

        
        
        while lower != solution.makespan {
            let start_time = Instant::now();
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
            hsr.simplify(&partial_solution, makespan_to_test);
            let partial_solution: PartialSolution =
            fur.simplify(&partial_solution, makespan_to_test);
            let partial_solution = finalize.simplify(&partial_solution, makespan_to_test);
            
            let mut var_assingment = None;
            //let mut simp_useful = true;
            //if simp_useful && instance.num_processors > 4 && instance.num_jobs > 20 {
            //    for _ in 0..5 {
            //        let random: Box<Precedence> = Box::new(Precedence::new(Box::new(PbNativeEncoder::new())));
            //        let mut random = Box::new(RandomEncoder::new(random, 2.0));
            //        random.basic_encode(&partial_solution, makespan_to_test);
            //        let clauses = random.output();
            //        let file_name = "./test";
            //        input_output::to_dimacs::print_to_dimacs(
            //            file_name,
            //            clauses,
            //            random.get_num_vars(),
            //        );
//
            //        var_assingment = self.sat_solver.as_ref().solve(file_name, 1);
            //        if var_assingment.is_some() {
            //            let old_bound: usize = solution.makespan;
            //            solution = random.decode(instance, var_assingment.as_ref().unwrap());
            //            let new_bound: usize = solution.makespan;
            //            if verbose {
            //            println!("SAT by simp");
            //            }
            //            if old_bound <= new_bound {
            //                println!("what, how did the solution get worse {}", solution);
            //            }
            //            break;
            //        }
            //    }
            //}
            //simp_useful = false;

            if var_assingment.is_none() {
                self.encoder
                    .basic_encode(&partial_solution, makespan_to_test);
                let clauses = self.encoder.output();
                let file_name: &str = "./test";
                input_output::to_dimacs::print_to_dimacs(
                    file_name,
                    clauses,
                    self.encoder.get_num_vars(),
                );
                let result = self.sat_solver.as_ref().solve(file_name, timeout);

                // TODO: there is definetly a better way of doing this, such that we get an any time algo, but i cba rn
                if result.is_timeout(){
                    return None;
                }
                
                var_assingment = result.unwrap();

                if var_assingment.is_none() {
                    lower = makespan_to_test + 1;
                    if verbose {
                    println!("UNSAT");
                    }
                } else {
                    let old_bound = solution.makespan;
                    solution = self
                        .encoder
                        .decode(instance, var_assingment.as_ref().unwrap());
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
            timeout -=  start_time.elapsed().as_secs_f64();
        }

        return Some(solution);
    }
}
