use crate::{
    common::timeout::Timeout,
    problem_instance::{problem_instance::ProblemInstance, solution::Solution},
};

pub trait Bound {
    fn bound(
        &self,
        problem: &ProblemInstance,
        lower_bound: usize,
        upper_bound: Option<Solution>,
        timeout: &Timeout,
    ) -> (usize, Option<Solution>);
}
