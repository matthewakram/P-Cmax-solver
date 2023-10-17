use crate::encoding::encoder::{Clause, VarNameGenerator};

use crate::bitvec::prelude::*;
use crate::bitvec::vec::BitVec;

#[derive(Clone, PartialEq, Eq)]
pub struct Node {
    pub var: usize,
    pub aux_var: usize,
    pub left_child: usize,
    pub right_child: usize,
}

pub struct BDD {
    pub nodes: Vec<Node>,
    pub root_num: usize,
}

pub fn leq(vars: &Vec<usize>, weights: &Vec<usize>, limit: usize) -> BDD {
    assert!(vars.len() > 1);
    let mut reachable: Vec<BitVec> = vec![];
    reachable.push(bitvec![0;weights[0]+1]);
    reachable[0].set(0, true);
    reachable[0].set(weights[0], true);

    // in reachable, you know which sum values are reachable after making a decision on i
    for i in 1..vars.len() {
        let size = (reachable[i - 1].len() + weights[i]).min(limit + 1);
        reachable.push(bitvec![0; size]);
        for r in 0..size {
            if (r < reachable[i - 1].len() && reachable[i - 1][r])
                || r >= weights[i] && reachable[i - 1][r - weights[i]]
            {
                reachable[i].set(r, true);
            }
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
    };
    let false_node = Node {
        var: 0,
        aux_var: 0,
        left_child: 0,
        right_child: 0,
    };

    nodes.push(false_node);
    nodes.push(true_node);

    // this will store the children of all elements at a given level (within limits)
    let mut reachable_i_nodes: Vec<usize> = vec![];
    for i in (0..vars.len()).rev() {
        let prev_reachable_i_nodes = reachable_i_nodes.clone();
        reachable_i_nodes = vec![0; limit + 1];
        // contain the nodes at this level, and the ids of these nodes
        let mut new_nodes: Vec<Node> = vec![];
        let mut new_node_ids: Vec<usize> = vec![];

        for reachable_i in 0..reachable[i].len() {
            if !reachable[i][reachable_i] {
                continue;
            }
            if i == vars.len() - 1 {
                reachable_i_nodes[reachable_i] = 1;
            } else {
                // The aux var is set to 0 temporarily for comparisons
                let new_node: Node = Node {
                    var: vars[i + 1],
                    aux_var: 0,
                    left_child: prev_reachable_i_nodes[reachable_i],
                    right_child: if reachable_i + weights[i + 1] < prev_reachable_i_nodes.len() {
                        prev_reachable_i_nodes[reachable_i + weights[i + 1]]
                    } else {
                        0
                    },
                };
                if new_node.left_child == new_node.right_child {
                    reachable_i_nodes[reachable_i] = new_node.left_child;
                    continue;
                }

                // if this newly created node has already been created, then there is no need to create a new node
                if new_nodes.len() == 0 || new_nodes[new_nodes.len() - 1] != new_node {
                    new_nodes.push(new_node.clone());
                    let node_id: usize = nodes.len();
                    new_node_ids.push(node_id);
                    nodes.push(new_node);
                    reachable_i_nodes[reachable_i] = node_id;
                } else {
                    let node_id = new_node_ids[new_node_ids.len() - 1];
                    reachable_i_nodes[reachable_i] = node_id;
                }
            }
        }
    }
    // all that is left is to push the root
    nodes.push(Node {
        var: vars[0],
        aux_var: 0,
        left_child: reachable_i_nodes[0],
        right_child: reachable_i_nodes[weights[0]],
    });

    //for node in &nodes {
    //    println!("var: {}  left: {}  right: {}", node.var, node.left_child, node.right_child);
    //}
    let root_index = nodes.len() - 1;
    return BDD {
        nodes: nodes,
        root_num: root_index,
    };
}

/// This is the same as leq, but if it sees one that fits perfectly, then it has to insert it
pub fn leq_greedy(vars: &Vec<usize>, weights: &Vec<usize>, limit: usize) -> BDD {
    assert!(vars.len() > 1);
    let mut reachable: Vec<BitVec> = vec![];
    reachable.push(bitvec![0;weights[0]+1]);
    reachable[0].set(0, true);
    reachable[0].set(weights[0], true);

    // in reachable, you know which sum values are reachable after making a decision on i

    // TODO: this way, we don't necessarily encode what I want, but the BDD is smaller. Lets see how this behaves
    for i in 1..vars.len() {
        let size = (reachable[i - 1].len() + weights[i]).min(limit + 1);
        reachable.push(bitvec![0; size]);
        for r in 0..size {

                if (r + weights[i] != limit && r < reachable[i - 1].len() && reachable[i - 1][r])
                    || (r >= weights[i] && reachable[i - 1][r - weights[i]])
                {
                    reachable[i].set(r, true);
                }
        }
    }
    //for a in &reachable{
    //    println!("{:?}", a);
    //}

    let mut nodes: Vec<Node> = vec![];

    let true_node = Node {
        var: 0,
        aux_var: 0,
        left_child: 1,
        right_child: 1,
    };
    let false_node = Node {
        var: 0,
        aux_var: 0,
        left_child: 0,
        right_child: 0,
    };

    nodes.push(false_node);
    nodes.push(true_node);

    // this will store the children of all elements at a given level (within limits)
    let mut reachable_i_nodes: Vec<usize> = vec![];
    for i in (0..vars.len()).rev() {
        let prev_reachable_i_nodes = reachable_i_nodes.clone();
        reachable_i_nodes = vec![0; limit + 1];
        // contain the nodes at this level, and the ids of these nodes
        let mut new_nodes: Vec<Node> = vec![];
        let mut new_node_ids: Vec<usize> = vec![];

        for reachable_i in 0..reachable[i].len() {
            if !reachable[i][reachable_i] {
                continue;
            }
            if i == vars.len() - 1 {
                reachable_i_nodes[reachable_i] = 1;
            } else {
                // The aux var is set to 0 temporarily for comparisons
                let new_node: Node = Node {
                    var: vars[i + 1],
                    aux_var: 0,
                    // TODO: This makes the BDD larger, but the logic easier, so how this behaves
                    //left_child: if reachable_i + weights[i+1] == limit {0} else {prev_reachable_i_nodes[reachable_i]},
                    left_child: prev_reachable_i_nodes[reachable_i],
                    right_child: if reachable_i + weights[i + 1] < prev_reachable_i_nodes.len() {
                        prev_reachable_i_nodes[reachable_i + weights[i + 1]]
                    } else {
                        0
                    },
                };
                if new_node.left_child == new_node.right_child {
                    reachable_i_nodes[reachable_i] = new_node.left_child;
                    continue;
                }

                // if this newly created node has already been created, then there is no need to create a new node
                if new_nodes.len() == 0 || new_nodes[new_nodes.len() - 1] != new_node {
                    new_nodes.push(new_node.clone());
                    let node_id: usize = nodes.len();
                    new_node_ids.push(node_id);
                    nodes.push(new_node);
                    reachable_i_nodes[reachable_i] = node_id;
                } else {
                    let node_id = new_node_ids[new_node_ids.len() - 1];
                    reachable_i_nodes[reachable_i] = node_id;
                }
            }
        }
    }
    // all that is left is to push the root
    nodes.push(Node {
        var: vars[0],
        aux_var: 0,
        left_child: reachable_i_nodes[0],
        right_child: reachable_i_nodes[weights[0]],
    });

    //for node in &nodes {
    //    println!("var: {}  left: {}  right: {}", node.var, node.left_child, node.right_child);
    //}
    let root_index = nodes.len() - 1;
    return BDD {
        nodes: nodes,
        root_num: root_index,
    };
}

pub fn assign_aux_vars(mut bdd: BDD, var_name_generator: &mut VarNameGenerator) -> BDD {
    for i in 0..bdd.nodes.len() {
        bdd.nodes[i].aux_var = var_name_generator.next();
    }
    return bdd;
}

pub fn encode(bdd: &BDD) -> Vec<Clause> {
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
        // TODO: play around with this
        clauses.push(Clause {
            vars: vec![
                // this is correct but makes it slower for some reason
                //(bdd.nodes[i].var as i32),
                -(bdd.nodes[i].aux_var as i32),
                (bdd.nodes[bdd.nodes[i].left_child].aux_var as i32),
            ],
        });
        // the second clause is -right_child_aux & node_var -> -node_aux
        clauses.push(Clause {
            vars: vec![
                -(bdd.nodes[i].var as i32),
                -(bdd.nodes[i].aux_var as i32),
                (bdd.nodes[bdd.nodes[i].right_child].aux_var as i32),
            ],
        });
    }
    return clauses;
}
