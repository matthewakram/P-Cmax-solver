use std::fmt::Display;




#[derive(Clone)]
pub struct Solution{
    pub makespan: usize,
    pub assignment: Vec<usize>,
}

impl Display for Solution{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "makespan: {}\nassignment: {:?}", self.makespan, self.assignment)
    }
}