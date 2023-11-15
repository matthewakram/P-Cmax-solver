#[cfg(test)]
mod tests {
    use crate::{bdd::bdd, common::timeout::Timeout};

    #[test]
    pub fn test_bdd_creation() {
        let _a = bdd::leq(&vec![0, 1, 2, 3, 4], &vec![1, 2, 3, 4, 5], &vec![49, 37, 21, 19, 7], 70, false, 0, &Timeout::new(10.0));
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
        let a = bdd::leq(&vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], &vec![1,2,3,4,5,6,7,8,9, 10], &vec![50, 45, 40, 35, 30, 25, 20, 15, 10, 5], 80, false, 0, &Timeout::new(10.0)).unwrap();
        let b = bdd::leq(&vec![3, 4, 5, 6, 7, 8, 9],&vec![4,5,6,7,8,9, 10], &vec![35, 30, 25, 20, 15, 10, 5], 80, false, 0, &Timeout::new(10.0)).unwrap();
        bdd::encode_bdd_bijective_relation(&a, &b);
    }

    //#[test]
    //pub fn test_dyn_bdd(){
    //    let p_i = ProblemInstance::new(3, 10, vec![35, 30, 25, 20, 15, 10, 5,3,2,2]);
    //    let part_sol = &crate::problem_instance::partial_solution::PartialSolution::new(p_i);
    //    let a = bdd_dyn::create_range_equivalency_table(part_sol, 45);
    //}
}
