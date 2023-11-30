

use crate::common::timeout::Timeout;
use crate::encoding::encoder::{Clause, VarNameGenerator, Clauses};

use crate::bitvec::prelude::*;
use crate::bitvec::vec::BitVec;

#[derive(Clone, PartialEq, Eq)]
pub struct Node {
    pub var: usize,
    pub aux_var: usize,
    pub left_child: usize,
    pub right_child: usize,
    pub job_num: usize,
    pub range: (usize, usize),
    //pub point_set: HashSet<usize>,
}

pub struct BDD {
    pub nodes: Vec<Node>,
    pub root_num: usize,
}

pub fn leq(
    jobs: &Vec<usize>,
    vars: &Vec<usize>,
    weights: &Vec<usize>,
    limit: usize,
    with_fur_nodes: bool,
    already_assigned: usize,
    timeout: &Timeout,
) -> Option<BDD> {
    //assert!(vars.len() > 1);
    assert_ne!(already_assigned, limit);
    let mut reachable: Vec<BitVec> = vec![];
    assert!(already_assigned + weights[0] <= limit);
    reachable.push(bitvec![0;already_assigned + weights[0]+1]);
    reachable[0].set(already_assigned, true);
    reachable[0].set(already_assigned + weights[0], true);

    // in reachable, you know which sum values are reachable after making a decision on i
    for i in 1..vars.len() {
        let size = (reachable[i - 1].len() + weights[i]).min(limit + 1);
        reachable.push(bitvec![0; size]);
        for r in already_assigned..size {
            if (r < reachable[i - 1].len() && reachable[i - 1][r])
                || r >= weights[i] && reachable[i - 1][r - weights[i]]
            {
                reachable[i].set(r, true);
            }
        }
        if timeout.time_finished() {
            return None;
        }
    }
    //for a in &reachable{
    //    println!("{:?}", a);
    //}

    // the nodes is a vector of all nodes, so that other nodes may reference them by their index
    let mut nodes: Vec<Node> = vec![];

    let true_node = Node {
        var: 0,
        aux_var: 0,
        left_child: 1,
        right_child: 1,
        job_num: usize::MAX,
        range: (0, usize::MAX),
        //point_set: HashSet::new(),
    };
    let false_node = Node {
        var: 0,
        aux_var: 0,
        left_child: 0,
        right_child: 0,
        job_num: usize::MAX,
        range: (0, usize::MAX),
        //point_set: HashSet::new(),
    };

    nodes.push(false_node);
    nodes.push(true_node);

    // this will store the children of all elements at a given level (within limits)
    let mut reachable_i_nodes: Vec<usize> = vec![];
    for i in (0..vars.len()).rev() {
        let prev_reachable_i_nodes = reachable_i_nodes.clone();
        reachable_i_nodes = vec![0; limit + 1];
        // contain the nodes at this level, and the ids of these nodes

        for reachable_i in already_assigned..reachable[i].len() {
            if !reachable[i][reachable_i] {
                continue;
            }
            if i == vars.len() - 1 {
                reachable_i_nodes[reachable_i] = 1;
            } else {
                let new_node: Node = Node {
                    var: vars[i + 1],
                    aux_var: 0,
                    left_child: prev_reachable_i_nodes[reachable_i],
                    right_child: if reachable_i + weights[i + 1] < prev_reachable_i_nodes.len() {
                        prev_reachable_i_nodes[reachable_i + weights[i + 1]]
                    } else {
                        0
                    },
                    job_num: jobs[i + 1],
                    range: (reachable_i, reachable_i),
                    //point_set: HashSet::from_iter(reachable_i..reachable_i+1),
                };
                if new_node.left_child == new_node.right_child {
                    reachable_i_nodes[reachable_i] = new_node.left_child;
                    continue;
                }

                // if this newly created node has already been created, then there is no need to create a new node
                if nodes[nodes.len() - 1].left_child != new_node.left_child
                    || nodes[nodes.len() - 1].right_child != new_node.right_child
                    || nodes[nodes.len() - 1].var != new_node.var
                    || (with_fur_nodes && reachable_i == limit - weights[i + 1])
                {
                    let node_id: usize = nodes.len();
                    nodes.push(new_node);
                    reachable_i_nodes[reachable_i] = node_id;
                } else {
                    let node_id = nodes.len() - 1;
                    reachable_i_nodes[reachable_i] = node_id;
                    nodes[node_id].range = (nodes[node_id].range.0, reachable_i);
                    //nodes[node_id].point_set.insert(reachable_i);
                }
            }
        }
        if timeout.time_finished() {
            return None;
        }
    }
    // all that is left is to push the root
    nodes.push(Node {
        var: vars[0],
        aux_var: 0,
        left_child: reachable_i_nodes[already_assigned + 0],
        right_child: reachable_i_nodes[already_assigned + weights[0]],
        job_num: jobs[0],
        range: (already_assigned, already_assigned),
        // !point_set: HashSet::from_iter(0..1),
    });

    //for node in &nodes {
    //    println!("var: {}  left: {}  right: {}, range: [{}, {}]", node.var, node.left_child, node.right_child, node.range.0, node.range.1);
    //}
    let root_index = nodes.len() - 1;
    return Some(BDD {
        nodes: nodes,
        root_num: root_index,
    });
}

/// This is the same as leq, but if it sees one that fits perfectly, then it has to insert it
//pub fn _leq_greedy(vars: &Vec<usize>, weights: &Vec<usize>, limit: usize) -> BDD {
//    assert!(vars.len() > 1);
//    let mut reachable: Vec<BitVec> = vec![];
//    reachable.push(bitvec![0;weights[0]+1]);
//    reachable[0].set(0, true);
//    reachable[0].set(weights[0], true);
//
//    // in reachable, you know which sum values are reachable after making a decision on i
//
//    // TODO: this way, we don't necessarily encode what I want, but the BDD is smaller. Lets see how this behaves
//    for i in 1..vars.len() {
//        let size = (reachable[i - 1].len() + weights[i]).min(limit + 1);
//        reachable.push(bitvec![0; size]);
//        for r in 0..size {
//            if (r + weights[i] != limit && r < reachable[i - 1].len() && reachable[i - 1][r])
//                || (r >= weights[i] && reachable[i - 1][r - weights[i]])
//            {
//                reachable[i].set(r, true);
//            }
//        }
//    }
//    //for a in &reachable{
//    //    println!("{:?}", a);
//    //}
//
//    let mut nodes: Vec<Node> = vec![];
//
//    let true_node = Node {
//        var: 0,
//        aux_var: 0,
//        left_child: 1,
//        right_child: 1,
//    };
//    let false_node = Node {
//        var: 0,
//        aux_var: 0,
//        left_child: 0,
//        right_child: 0,
//    };
//
//    nodes.push(false_node);
//    nodes.push(true_node);
//
//    // this will store the children of all elements at a given level (within limits)
//    let mut reachable_i_nodes: Vec<usize> = vec![];
//    for i in (0..vars.len()).rev() {
//        let prev_reachable_i_nodes = reachable_i_nodes.clone();
//        reachable_i_nodes = vec![0; limit + 1];
//        // contain the nodes at this level, and the ids of these nodes
//        let mut new_nodes: Vec<Node> = vec![];
//        let mut new_node_ids: Vec<usize> = vec![];
//
//        for reachable_i in 0..reachable[i].len() {
//            if !reachable[i][reachable_i] {
//                continue;
//            }
//            if i == vars.len() - 1 {
//                reachable_i_nodes[reachable_i] = 1;
//            } else {
//                // The aux var is set to 0 temporarily for comparisons
//                let new_node: Node = Node {
//                    var: vars[i + 1],
//                    aux_var: 0,
//                    // TODO: This makes the BDD larger, but the logic easier, so how this behaves
//                    //left_child: if reachable_i + weights[i+1] == limit {0} else {prev_reachable_i_nodes[reachable_i]},
//                    left_child: prev_reachable_i_nodes[reachable_i],
//                    right_child: if reachable_i + weights[i + 1] < prev_reachable_i_nodes.len() {
//                        prev_reachable_i_nodes[reachable_i + weights[i + 1]]
//                    } else {
//                        0
//                    },
//                };
//                if new_node.left_child == new_node.right_child {
//                    reachable_i_nodes[reachable_i] = new_node.left_child;
//                    continue;
//                }
//
//                // if this newly created node has already been created, then there is no need to create a new node
//                if new_nodes.len() == 0 || new_nodes[new_nodes.len() - 1] != new_node {
//                    new_nodes.push(new_node.clone());
//                    let node_id: usize = nodes.len();
//                    new_node_ids.push(node_id);
//                    nodes.push(new_node);
//                    reachable_i_nodes[reachable_i] = node_id;
//                } else {
//                    let node_id = new_node_ids[new_node_ids.len() - 1];
//                    reachable_i_nodes[reachable_i] = node_id;
//                }
//            }
//        }
//    }
//    // all that is left is to push the root
//    nodes.push(Node {
//        var: vars[0],
//        aux_var: 0,
//        left_child: reachable_i_nodes[0],
//        right_child: reachable_i_nodes[weights[0]],
//    });
//
//    //for node in &nodes {
//    //    println!("var: {}  left: {}  right: {}", node.var, node.left_child, node.right_child);
//    //}
//    let root_index = nodes.len() - 1;
//    return BDD {
//        nodes: nodes,
//        root_num: root_index,
//    };
//}

pub fn assign_aux_vars(mut bdd: BDD, var_name_generator: &mut VarNameGenerator) -> BDD {
    // what we have here is this property. If your left node is not a right node of another node,
    // then it can get the same var as you
    //let mut is_right_node = bitvec![0;bdd.nodes.len()];
    //for i in 0..bdd.nodes.len() {
    //    is_right_node.set(bdd.nodes[i].right_child, true);
    //}
    //let mut is_left_node = bitvec![0;bdd.nodes.len()];
    //let mut is_left_node_twice = bitvec![0;bdd.nodes.len()];
    //for i in 0..bdd.nodes.len() {
    //    if is_left_node[bdd.nodes[i].left_child] {
    //        is_left_node_twice.set(bdd.nodes[i].left_child, true);
    //    } else {
    //        is_left_node.set(bdd.nodes[i].left_child, true);
    //    }
    //}

    for i in (0..bdd.nodes.len()).rev() {
        if bdd.nodes[i].aux_var == 0 {
            bdd.nodes[i].aux_var = var_name_generator.next();
        }
        //let left = bdd.nodes[i].left_child;
        //if bdd.nodes[left].aux_var == 0 && !is_right_node[left] && !is_left_node_twice[left] {
        //    let a: usize = bdd.nodes[i].left_child;
        //    bdd.nodes[a].aux_var = bdd.nodes[i].aux_var;
        //}
    }
    return bdd;
}

pub fn _encode_bad(bdd: &BDD) -> Clauses {
    let mut clauses: Clauses = Clauses::new();

    // we first have to handle that false and true nodes, as well as the root node
    clauses.add_clause(Clause {
        vars: vec![-(bdd.nodes[0].aux_var as i32)],
    });
    clauses.add_clause(Clause {
        vars: vec![bdd.nodes[1].aux_var as i32],
    });
    clauses.add_clause(Clause {
        vars: vec![bdd.nodes[bdd.root_num].aux_var as i32],
    });
    // now we handle the two clauses per node
    for i in 2..bdd.nodes.len() {
        // the first clause is -left_child_aux -> -node_aux
        clauses.add_clause(Clause {
            vars: vec![
                // this is correct but makes it slower for some reason
                (bdd.nodes[i].var as i32),
                -(bdd.nodes[i].aux_var as i32),
                (bdd.nodes[bdd.nodes[i].left_child].aux_var as i32),
            ],
        });
        // the second clause is -right_child_aux & node_var -> -node_aux

        clauses.add_clause(Clause {
            vars: vec![
                -(bdd.nodes[i].var as i32),
                -(bdd.nodes[i].aux_var as i32),
                (bdd.nodes[bdd.nodes[i].right_child].aux_var as i32),
            ],
        });

    }

    return clauses;
}

pub fn _encode_bad_new(bdd: &BDD) -> Vec<Clause> {
    let mut clauses: Vec<Clause> = vec![];

    // we first have to handle that false and true nodes, as well as the root node
    clauses.push(Clause {
        vars: vec![-(bdd.nodes[0].aux_var as i32)],
    });
    clauses.push(Clause {
        vars: vec![bdd.nodes[1].aux_var as i32],
    });
    clauses.push(Clause {
        vars: vec![bdd.nodes[bdd.root_num].aux_var as i32],
    });
    // now we handle the two clauses per node
    for i in 2..bdd.nodes.len() {
        // the first clause is -left_child_aux -> -node_aux
        clauses.push(Clause {
            vars: vec![
                // this is correct but makes it slower for some reason
                (bdd.nodes[i].var as i32),
                -(bdd.nodes[i].aux_var as i32),
                (bdd.nodes[bdd.nodes[i].left_child].aux_var as i32),
            ],
        });

        clauses.push(Clause {
            vars: vec![
                // this is correct but makes it slower for some reason
                (bdd.nodes[i].var as i32),
                (bdd.nodes[i].aux_var as i32),
                -(bdd.nodes[bdd.nodes[i].left_child].aux_var as i32),
            ],
        });

        clauses.push(Clause {
            vars: vec![
                // this is correct but makes it slower for some reason
                -(bdd.nodes[i].var as i32),
                -(bdd.nodes[i].aux_var as i32),
                (bdd.nodes[bdd.nodes[i].right_child].aux_var as i32),
            ],
        });

        clauses.push(Clause {
            vars: vec![
                // this is correct but makes it slower for some reason
                -(bdd.nodes[i].var as i32),
                (bdd.nodes[i].aux_var as i32),
                -(bdd.nodes[bdd.nodes[i].right_child].aux_var as i32),
            ],
        });

        clauses.push(Clause {
            vars: vec![
                // this is correct but makes it slower for some reason
                (bdd.nodes[bdd.nodes[i].left_child].aux_var as i32),
                (bdd.nodes[bdd.nodes[i].right_child].aux_var as i32),
                -(bdd.nodes[i].aux_var as i32),
            ],
        });

        clauses.push(Clause {
            vars: vec![
                // this is correct but makes it slower for some reason
                -(bdd.nodes[bdd.nodes[i].left_child].aux_var as i32),
                -(bdd.nodes[bdd.nodes[i].right_child].aux_var as i32),
                (bdd.nodes[i].aux_var as i32),
            ],
        });
    }

    return clauses;
}

pub fn encode(bdd: &BDD) -> Clauses {
    let mut clauses: Clauses = Clauses::new();

    // we first have to handle that false and true nodes, as well as the root node
    clauses.add_clause(Clause {
        vars: vec![-(bdd.nodes[0].aux_var as i32)],
    });
    clauses.add_clause(Clause {
        vars: vec![bdd.nodes[1].aux_var as i32],
    });
    clauses.add_clause(Clause {
        vars: vec![bdd.nodes[bdd.root_num].aux_var as i32],
    });
    // now we handle the two clauses per node
    for i in 2..bdd.nodes.len() {
        // the first clause is -left_child_aux -> -node_aux
        if bdd.nodes[i].aux_var != bdd.nodes[bdd.nodes[i].left_child].aux_var {
            clauses.add_clause(Clause {
                vars: vec![
                    -(bdd.nodes[i].aux_var as i32),
                    (bdd.nodes[bdd.nodes[i].left_child].aux_var as i32),
                ],
            });
        }
        if bdd.nodes[bdd.nodes[i].left_child].aux_var == bdd.nodes[bdd.nodes[i].right_child].aux_var
        {
            if bdd.nodes.len() != 3 {
                for node in &bdd.nodes {
                    println!(
                        "job num {} left {} right {} aux {}",
                        node.job_num, node.left_child, node.right_child, node.aux_var
                    );
                }
                println!("error occured at node {}", i);
            }
            assert!(bdd.nodes.len() == 3);
        }

        // the second clause is -right_child_aux & node_var -> -node_aux
        clauses.add_clause(Clause {
            vars: vec![
                -(bdd.nodes[i].var as i32),
                -(bdd.nodes[i].aux_var as i32),
                (bdd.nodes[bdd.nodes[i].right_child].aux_var as i32),
            ],
        });
    }
    return clauses;
}

//pub fn _eq(vars: &Vec<usize>, weights: &Vec<usize>, limit: usize) -> Option<BDD> {
//    assert!(vars.len() > 1);
//    let mut reachable: Vec<BitVec> = vec![];
//    reachable.push(bitvec![0;weights[0]+1]);
//    reachable[0].set(0, true);
//    reachable[0].set(weights[0], true);
//
//    // in reachable, you know which sum values are reachable after making a decision on i
//    for i in 1..vars.len() {
//        let size = (reachable[i - 1].len() + weights[i]).min(limit + 1);
//        reachable.push(bitvec![0; size]);
//
//        // with this we ensure that the BDD is only true iff we reach limit exactly
//        if i == vars.len() - 1 {
//            if (limit < reachable[i - 1].len() && reachable[i - 1][limit])
//                || limit >= weights[i] && reachable[i - 1][limit - weights[i]]
//            {
//                reachable[i].set(limit, true);
//                continue;
//            } else {
//                return None;
//            }
//        }
//        for r in 0..size {
//            if (r < reachable[i - 1].len() && reachable[i - 1][r])
//                || r >= weights[i] && reachable[i - 1][r - weights[i]]
//            {
//                reachable[i].set(r, true);
//            }
//        }
//    }
//    //for a in &reachable{
//    //    println!("{:?}", a);
//    //}
//
//    // the nodes is a vector of all nodes, so that other nodes may reference them by their index
//    let mut nodes: Vec<Node> = vec![];
//
//    let true_node = Node {
//        var: 0,
//        aux_var: 0,
//        left_child: 1,
//        right_child: 1,
//    };
//    let false_node = Node {
//        var: 0,
//        aux_var: 0,
//        left_child: 0,
//        right_child: 0,
//    };
//
//    nodes.push(false_node);
//    nodes.push(true_node);
//
//    // this will store the children of all elements at a given level (within limits)
//    let mut reachable_i_nodes: Vec<usize> = vec![];
//    for i in (0..vars.len()).rev() {
//        let prev_reachable_i_nodes = reachable_i_nodes.clone();
//        reachable_i_nodes = vec![0; limit + 1];
//        // contain the nodes at this level, and the ids of these nodes
//        let mut new_nodes: Vec<Node> = vec![];
//        let mut new_node_ids: Vec<usize> = vec![];
//
//        for reachable_i in 0..reachable[i].len() {
//            if !reachable[i][reachable_i] {
//                continue;
//            }
//            if i == vars.len() - 1 {
//                reachable_i_nodes[reachable_i] = 1;
//            } else {
//                // The aux var is set to 0 temporarily for comparisons
//                let new_node: Node = Node {
//                    var: vars[i + 1],
//                    aux_var: 0,
//                    left_child: prev_reachable_i_nodes[reachable_i],
//                    right_child: if reachable_i + weights[i + 1] < prev_reachable_i_nodes.len() {
//                        prev_reachable_i_nodes[reachable_i + weights[i + 1]]
//                    } else {
//                        0
//                    },
//                };
//                if new_node.left_child == new_node.right_child {
//                    reachable_i_nodes[reachable_i] = new_node.left_child;
//                    continue;
//                }
//
//                // if this newly created node has already been created, then there is no need to create a new node
//                if new_nodes.len() == 0 || new_nodes[new_nodes.len() - 1] != new_node {
//                    new_nodes.push(new_node.clone());
//                    let node_id: usize = nodes.len();
//                    new_node_ids.push(node_id);
//                    nodes.push(new_node);
//                    reachable_i_nodes[reachable_i] = node_id;
//                } else {
//                    let node_id = new_node_ids[new_node_ids.len() - 1];
//                    reachable_i_nodes[reachable_i] = node_id;
//                }
//            }
//        }
//    }
//    // all that is left is to push the root
//    nodes.push(Node {
//        var: vars[0],
//        aux_var: 0,
//        left_child: reachable_i_nodes[0],
//        right_child: reachable_i_nodes[weights[0]],
//    });
//
//    for node in &nodes {
//        println!(
//            "var: {}  left: {}  right: {}",
//            node.var, node.left_child, node.right_child
//        );
//    }
//    let root_index = nodes.len() - 1;
//    return Some(BDD {
//        nodes: nodes,
//        root_num: root_index,
//    });
//}

pub fn encode_bdd_bijective_relation(bdd1: &BDD, bdd2: &BDD) -> Clauses {
    // bijection says that the node i in bdd1 is equivalent to the variable bijection[i] in bdd2
    let mut bijection: Vec<usize> = vec![0, 1];
    let mut bdd2_i = 2;
    //for i in 0..bdd1.nodes.len() {
    //    let node = &bdd1.nodes[i];
    //    println!(
    //        "i: {}var: {}  left: {}  right: {}",
    //        i, node.var, node.left_child, node.right_child
    //    );
    //}
    //for i in 0..bdd2.nodes.len() {
    //    let node = &bdd2.nodes[i];
    //    println!(
    //        "i: {} var: {}  left: {}  right: {}",
    //        i, node.var, node.left_child, node.right_child
    //    );
    //}
    //let mut num_bis = 0;
    for bdd1_i in 2..bdd1.nodes.len() {
        while bdd2_i != bdd2.nodes.len()
            && (bdd2.nodes[bdd2_i].job_num > bdd1.nodes[bdd1_i].job_num
                || (bdd2.nodes[bdd2_i].range.0 < bdd1.nodes[bdd1_i].range.0
                    && bdd2.nodes[bdd2_i].job_num == bdd1.nodes[bdd1_i].job_num))
        {
            bdd2_i += 1;
        }

        if bdd2_i != bdd2.nodes.len()
            && bdd2.nodes[bdd2_i].job_num == bdd1.nodes[bdd1_i].job_num
            && bdd2.nodes[bdd2_i].range.0 >= bdd1.nodes[bdd1_i].range.0
            && bdd2.nodes[bdd2_i].range.1 <= bdd1.nodes[bdd1_i].range.1
        // TODO: More tests on which one is better, but I will stick with this for now
        // TODO: I can prove that this is not necessary, but for now it just makes me feel better
        //&& bdd2.nodes[bdd2_i].point_set.is_subset(&bdd1.nodes[bdd1_i].point_set)
        //&& bdd2.nodes[bdd2_i].point_set.len() == 1
        {
            bijection.push(bdd2_i);
            //num_bis += 1;
            bdd2_i += 1;
        } else {
            bijection.push(usize::MAX);
        }
    }
    //println!("{:?}", bijection);
    //println!("number of bijections found {}/{}", num_bis, bdd2.nodes.len());

    let mut clauses = Clauses::new();
    for bdd1_node in 2..bdd1.nodes.len() {
        if bijection[bdd1_node] == usize::MAX {
            continue;
        }
        let node: &Node = &bdd1.nodes[bdd1_node];
        let equiv_node = &bdd2.nodes[bijection[bdd1_node]];
        // this is reachable -> (that is reachable -> that is false)
        // this means that when multiple processors are on an equivalent value, that it will only try to insert it into the first one
        clauses.add_clause(Clause {
            vars: vec![
                -(node.aux_var as i32),
                -(equiv_node.aux_var as i32),
                -(equiv_node.var as i32),
            ],
        });
    }
    return clauses;
}
