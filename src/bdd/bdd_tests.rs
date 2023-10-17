#[cfg(test)]
mod tests {
    use crate::bdd::bdd;

    #[test]
    pub fn test_bdd_creation() {
        let _a = bdd::leq(&vec![1, 2, 3, 4, 5], &vec![49, 37, 21, 19, 7], 70);
    }

    #[test]
    pub fn test_bdd_greedy_creation() {
        let _a = bdd::leq_greedy(&vec![1, 2, 3, 4, 5], &vec![49, 37, 21, 19, 7], 70);
    }
}
