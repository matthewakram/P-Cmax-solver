use super::makespan_scheduler::MakespanScheduler;

#[derive(Clone)]
pub struct LinearMakespan {}

impl MakespanScheduler for LinearMakespan {
    fn next_makespan(
        &mut self,
        _: &crate::problem_instance::problem_instance::ProblemInstance,
        _: &crate::problem_instance::solution::Solution,
        _: usize,
        upper: usize,
    ) -> usize {
        return upper - 1;
    }
}
