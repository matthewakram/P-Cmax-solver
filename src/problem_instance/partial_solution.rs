use super::problem_instance::ProblemInstance;


#[derive(Clone)]
pub struct PartialSolution{
    pub instance: ProblemInstance,
    pub possible_allocations: Vec<Vec<usize>>
}