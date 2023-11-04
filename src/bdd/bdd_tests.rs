#[cfg(test)]
mod tests {
    use crate::bdd::bdd;

    #[test]
    pub fn test_bdd_creation() {
        let _a = bdd::leq(&vec![0, 1, 2, 3, 4], &vec![1, 2, 3, 4, 5], &vec![49, 37, 21, 19, 7], 70, false);
    }

    #[test]
    pub fn test_bdd_greedy_creation() {
        //let _a = bdd::_leq_greedy(&vec![1, 2, 3, 4, 5], &vec![49, 37, 21, 19, 7], 70);
    }

    #[test]
    pub fn test_bdd_eq_creation() {
        //let _a = bdd::_eq(&vec![1, 2, 3, 4, 5], &vec![49, 37, 21, 19, 7], 70);
    }

    #[test]
    pub fn test_bdd_bij() {
        let a = bdd::leq(&vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], &vec![1,2,3,4,5,6,7,8,9, 10], &vec![50, 45, 40, 35, 30, 25, 20, 15, 10, 5], 80, false);
        let b = bdd::leq(&vec![3, 4, 5, 6, 7, 8, 9],&vec![4,5,6,7,8,9, 10], &vec![35, 30, 25, 20, 15, 10, 5], 80, false);
        bdd::encode_bdd_bijective_relation(&a, &b);
    }
}
