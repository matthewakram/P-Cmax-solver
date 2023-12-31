#[cfg(test)]
mod tests {

    use crate::{solvers::{branch_and_bound::branch_and_bound::BranchAndBound, solver_manager::SolverManager}, bounds::{upper_bounds::{lpt::{self}, lptp, lptpp}, bound::Bound, lower_bounds::{pigeon_hole, max_job_size, fs, sss_bound_tightening, middle}}};


    #[test]
    pub fn b_and_b_test(){
        use crate::{problem_instance::problem_instance::ProblemInstance, common::timeout::Timeout};

        let sizes:Vec<usize> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 14,12, 13, 4,3,2,4,6,54,43,12,32, 54,12, 12,43,6,7,3,3];
        let instance = ProblemInstance::new(5, sizes.len(), sizes);
        let timeout = Timeout::new(100.0);
        let bounds: Vec<Box<dyn Bound>> = vec![
            Box::new(pigeon_hole::PigeonHole {}),
            Box::new(max_job_size::MaxJobSize {}),
            Box::new(middle::MiddleJobs {}),
            Box::new(fs::FeketeSchepers {}),
            Box::new(lpt::LPT {}),
            Box::new(lptp::Lptp {}),
            Box::new(sss_bound_tightening::SSSBoundStrengthening {}),
            Box::new(lptpp::Lptpp {}),
        ];
    
        let (mut lower_bound, mut upper_bound) = (1, None);
        //TODO; make this dynamic
        for i in 0..bounds.len() {
            let bound = &bounds[i];
            (lower_bound, upper_bound) =
                bound.bound(&instance, lower_bound, upper_bound, &timeout);
            println!(
                "lower: {} upper {}",
                lower_bound,
                if upper_bound.is_some() {
                    upper_bound.as_ref().unwrap().makespan
                } else {
                    0
                }
            );
            if timeout.time_finished()
                || (upper_bound.is_some() && upper_bound.as_ref().unwrap().makespan == lower_bound)
            {
                break;
            }
        }
        
        let mut upper = upper_bound.unwrap();
        upper.makespan *=2;
        
        let mut solver = BranchAndBound::new();
        let sol = solver.solve(&instance, 5, &upper, &timeout, false);
        assert!(sol.is_some());
        let sol = sol.unwrap();
        println!("{}", sol);
    }
}