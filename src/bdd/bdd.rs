
use crate::encoding::encoder::VarNameGenerator;

#[derive(Clone, PartialEq, Eq)]
pub struct Node {
    var: usize,
    aux_var: usize,
    left_child: usize,
    right_child: usize,
}

pub struct BDD{
    pub nodes : Vec<Node>,
    pub root_num: usize
}

pub fn leq(
    vars: Vec<usize>,
    weights: Vec<usize>,
    limit: usize,
    var_name_generator: &mut VarNameGenerator,
) -> BDD {
    assert!(vars.len() > 1);
    let mut reachable: Vec<Vec<usize>> = vec![];
    reachable.push(vec![0, weights[0]]);

    // in reachable, you know which sum values are reachable after making a decision on i
    for i in 1..vars.len() {
        reachable.push(
            (0..limit + 1)
                .filter(|j| {
                    reachable[i - 1].contains(j)
                        || (*j >= weights[i] && reachable[i - 1].contains(&(j - weights[i])))
                })
                .collect(),
        );
    }

    // the nodes is a vector of all nodes, so that other nodes may reference them by their index
    let mut nodes: Vec<Node> = vec![];

    let true_node = Node {
        var: 0,
        aux_var: var_name_generator.next(),
        left_child: 1,
        right_child: 1,
    };
    let false_node = Node {
        var: 0,
        aux_var: var_name_generator.next(),
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
        let mut new_nodes = vec![];
        let mut new_node_ids = vec![];

        for reachable_i in &reachable[i] {
            if i == vars.len()-1 {
                reachable_i_nodes[*reachable_i] = 1;
            } else {
                // The aux var is set to 0 temporarily for comparisons
                let new_node: Node = Node {
                    var: vars[i+1],
                    aux_var: 0,
                    left_child: prev_reachable_i_nodes[*reachable_i],
                    right_child: if *reachable_i + weights[i+1] < prev_reachable_i_nodes.len() {
                        prev_reachable_i_nodes[*reachable_i + weights[i+1]]
                    } else {
                        0
                    },
                };
                if new_node.left_child == new_node.right_child {
                    reachable_i_nodes[*reachable_i] = new_node.left_child;
                    continue;
                }

                // if this newly created node has already been created, then there is no need to create a new node
                let node_id = new_nodes.iter().position(|r| r == &new_node);
                if node_id.is_none() {
                    new_nodes.push(new_node.clone());
                    let node_id: usize = nodes.len();
                    new_node_ids.push(node_id);
                    nodes.push(new_node);
                    reachable_i_nodes[*reachable_i] = node_id;
                } else {
                    let node_id = new_node_ids[node_id.unwrap()];
                    reachable_i_nodes[*reachable_i] = node_id;
                }
            }
        }
    }
    // all that is left is to push the root
    // TODO: ensure that there is more than one var
    nodes.push(Node { var: vars[0], aux_var: 0, left_child: reachable_i_nodes[0], right_child: reachable_i_nodes[weights[0]] });

    //for node in &nodes {
    //    println!("var: {}  left: {}  right: {}", node.var, node.left_child, node.right_child);
    //}
    let root_index = nodes.len() - 1;
    return BDD { nodes: nodes, root_num: root_index }
}
