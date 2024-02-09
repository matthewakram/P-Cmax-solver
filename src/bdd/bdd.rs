use crate::common::timeout::Timeout;
use crate::encoding::sat_encoder::{Clause, Clauses, VarNameGenerator};

use crate::bitvec::prelude::*;
use crate::bitvec::vec::BitVec;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node {
    pub var: usize,
    pub aux_var: usize,
    pub left_child: usize,
    pub right_child: usize,
    pub job_num: usize,
    pub range: (usize, usize),
}

#[derive(Debug)]
pub struct BDD {
    pub nodes: Vec<Node>,
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
    assert!(already_assigned < limit);
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
    };
    let false_node = Node {
        var: 0,
        aux_var: 0,
        left_child: 0,
        right_child: 0,
        job_num: usize::MAX,
        range: (0, usize::MAX),
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
    });

    //for node in &nodes {
    //    println!("var: {}  left: {}  right: {}, range: [{}, {}]", node.var, node.left_child, node.right_child, node.range.0, node.range.1);
    //}
    return Some(BDD { nodes: nodes });
}

pub fn assign_aux_vars(mut bdd: BDD, var_name_generator: &mut VarNameGenerator) -> BDD {
    for i in (0..bdd.nodes.len()).rev() {
        if bdd.nodes[i].aux_var == 0 {
            bdd.nodes[i].aux_var = var_name_generator.next();
        }
    }
    return bdd;
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
        vars: vec![bdd.nodes[bdd.nodes.len() - 1].aux_var as i32],
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
