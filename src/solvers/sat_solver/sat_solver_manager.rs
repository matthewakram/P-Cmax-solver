use crate::{
    makespan_scheduling::makespan_scheduler::MakespanScheduler,
    problem_instance::{problem_instance::ProblemInstance, partial_solution::PartialSolution, solution::Solution}, solvers::solver::SatSolver, encoding::encoder::Encoder, input_output, problem_simplification::{half_size_rule::HalfSizeRule, simplification_rule::SimpRule, fill_up_rule::FillUpRule},
};

pub struct SatSolverManager {
    pub sat_solver: Box<dyn SatSolver>,
    pub makespan_scheduler: Box<dyn MakespanScheduler>,
    pub encoder: Box<dyn Encoder>,
}

impl SatSolverManager {
    pub fn solve(&mut self, instance: &ProblemInstance, lower: usize, upper: &Solution) -> Solution {

        let mut solution: Solution = upper.clone();
        let mut lower = lower;

        while lower != solution.makespan {
            let makespan_to_test = self.makespan_scheduler.next_makespan(instance, &solution, lower, solution.makespan);
            println!("makespan to test: {}", makespan_to_test);

            // Here We refine the solution that we have
            let partial_solution = PartialSolution::new(instance.clone());
            let mut hsr = HalfSizeRule{};
            let mut fur = FillUpRule{};
            let partial_solution: PartialSolution = hsr.simplify(&partial_solution, makespan_to_test);
            let partial_solution: PartialSolution = fur.simplify(&partial_solution, makespan_to_test);


            self.encoder.basic_encode(&partial_solution, makespan_to_test);
            let clauses = self.encoder.output();
            let file_name = "./test";
            input_output::to_dimacs::print_to_dimacs(file_name, clauses, self.encoder.get_num_vars());
            let var_assingment = self.sat_solver.as_ref().solve(file_name);
            if var_assingment.is_none() {
                lower = makespan_to_test +1;
                println!("UNSAT");
            }else {
                let old_bound = solution.makespan;
                solution = self.encoder.decode(instance, var_assingment.as_ref().unwrap());
                let new_bound = solution.makespan;
                println!("SAT");
                if old_bound <= new_bound {
                    println!("what, how did the solution get worse {}", solution);
                    break;
                }
            }
            println!("lower: {} upper {}", lower, solution.makespan);
        }
        
        return solution;
    }
}
