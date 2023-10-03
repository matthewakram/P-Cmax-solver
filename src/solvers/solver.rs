
pub trait SatSolver {
    fn solve(&self, file_name: &str) -> Option<Vec<i32>>;
}