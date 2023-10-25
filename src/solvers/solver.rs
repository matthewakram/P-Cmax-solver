
pub trait SatSolver {
    fn solve(&self, file_name: &str, timeout: u64) -> Option<Vec<i32>>;
}