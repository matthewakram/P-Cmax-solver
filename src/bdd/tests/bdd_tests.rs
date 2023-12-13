#[cfg(test)]
mod tests {
    use rand::{rngs::ThreadRng, Rng};

    use crate::{
        bdd::{
            bdd,
            bdd_dyn::{self, RangeTable},
        },
        common::timeout::Timeout,
    };

    #[test]
    pub fn test_bdd_creation() {
        let a = bdd::leq(
            &vec![0, 1, 2, 3, 4],
            &vec![1, 2, 3, 4, 5],
            &vec![49, 37, 21, 19, 7],
            70,
            false,
            0,
            &Timeout::new(10.0),
        )
        .unwrap();
    }

    #[test]
    pub fn test_bdd_accuracy() {
        let mut rng = ThreadRng::default();
        for i in 1..30 {
            let max: usize = i * rng.gen_range(1..100);
            let job_sizes: Vec<usize> = (0..i)
                .into_iter()
                .map(|_| rng.gen_range(1..100.min(max + 1)))
                .collect::<Vec<usize>>();
            let bdd = bdd::leq(
                &(0..i).into_iter().collect::<Vec<usize>>(),
                &(1..i + 1).into_iter().collect::<Vec<usize>>(),
                &job_sizes,
                max,
                false,
                0,
                &Timeout::new(5.0),
            )
            .unwrap();

            for _ in 0..20 {
                let mut sum = 0;
                let mut current_node = bdd.nodes.len() - 1;
                while sum <= max && current_node > 1 {
                    let take: bool = rng.gen_bool(0.5);
                    if take {
                        sum += job_sizes[bdd.nodes[current_node].job_num];
                        current_node = bdd.nodes[current_node].right_child;
                    } else {
                        current_node = bdd.nodes[current_node].left_child;
                    }
                }
                if current_node == 1 {
                    assert!(sum <= max);
                } else {
                    assert!(sum > max && current_node == 0);
                }
            }
        }
    }

    #[test]
    pub fn test_bdd_dyn_accuracy() {
        let mut rng = ThreadRng::default();
        for i in 1..60 {
            let max: usize = i * rng.gen_range(1..100);
            let job_sizes: Vec<usize> = (0..i)
                .into_iter()
                .map(|_| rng.gen_range(1..100.min(max + 1)))
                .collect::<Vec<usize>>();
            let jobs = (0..i).into_iter().collect::<Vec<usize>>();
            let range_table = RangeTable::new(&jobs, &job_sizes, max);
            let bdd = bdd_dyn::DynBDD::leq(
                &jobs,
                &(1..i + 1).into_iter().collect::<Vec<usize>>(),
                &job_sizes,
                max,
                0,
                &range_table,
                &Timeout::new(5.0),
            )
            .unwrap();

            for _ in 0..40 {
                let mut sum = 0;
                let mut current_node = bdd.nodes.len() - 1;
                while sum <= max && current_node > 1 {
                    let take: bool = rng.gen_bool(0.5);
                    if take {
                        sum += job_sizes[bdd.nodes[current_node].job_num];
                        current_node = bdd.nodes[current_node].right_child;
                    } else {
                        current_node = bdd.nodes[current_node].left_child;
                    }
                }
                if current_node == 1 {
                    assert!(sum <= max);
                } else {
                    assert!(sum > max && current_node == 0);
                }
            }
        }
    }

    //#[test]
    //pub fn test_dyn_bdd(){
    //    let p_i = ProblemInstance::new(3, 10, vec![35, 30, 25, 20, 15, 10, 5,3,2,2]);
    //    let part_sol = &crate::problem_instance::partial_solution::PartialSolution::new(p_i);
    //    let a = bdd_dyn::create_range_equivalency_table(part_sol, 45);
    //}
}
