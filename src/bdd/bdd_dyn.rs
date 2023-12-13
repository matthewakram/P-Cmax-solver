use std::collections::HashMap;

use bitvec::{prelude::*, vec::BitVec};

use crate::{
    common::timeout::Timeout,
    encoding::encoder::{Clause, Clauses, VarNameGenerator},
    problem_instance::partial_solution::PartialSolution,
};

#[derive(Debug)]
pub struct DynNode {
    pub var: usize,
    pub aux_var: usize,
    pub range_num: usize,
    pub job_num: usize,
    pub range: (usize, usize),
    pub left_child: usize,
    pub right_child: usize,
}

#[derive(Debug)]
pub struct RangeTable {
    ranges: Vec<Vec<usize>>,
    job_position: HashMap<usize, usize>,
    range_sizes: Vec<(usize, usize)>,
    makespan: usize,
}

#[derive(Debug)]
pub struct DynBDD {
    pub nodes: Vec<DynNode>,
}

impl RangeTable {
    pub fn get_range(&self, job_num: usize, value: usize) -> Option<usize> {
        let index = self.job_position.get(&job_num);
        if index.is_none() {
            return None;
        }

        let index = *index.unwrap();
        return Some(self.ranges[index][value]);
    }

    pub fn new(jobs: Vec<usize>, job_sizes: Vec<usize>, makespan: usize) -> RangeTable {
        let mut ranges: Vec<Vec<usize>> = vec![vec![0; makespan + 1]; job_sizes.len() - 1];
        ranges.push(vec![0; makespan - job_sizes[job_sizes.len() - 1] + 1]);
        ranges[job_sizes.len() - 1].append(&mut vec![1; job_sizes[job_sizes.len() - 1]]);
        let mut range_sizes: Vec<(usize, usize)> = vec![];
        range_sizes.push((0, makespan - job_sizes[job_sizes.len() - 1]));
        range_sizes.push((makespan - job_sizes[job_sizes.len() - 1] + 1, makespan));

        let mut range_num = 2;
        for job in (0..job_sizes.len() - 1).rev() {
            let job_size = job_sizes[job];
            let mut previous_left = ranges[job + 1][job_size];
            let mut previous_right = ranges[job + 1][0];
            let mut last_range_end: usize = 0;
            for i in 0..makespan + 1 {
                let new_right = if i + 1 + job_size > makespan {
                    usize::MAX
                } else {
                    ranges[job + 1][i + job_size + 1]
                };
                let new_left = if i + 1 > makespan {
                    usize::MAX
                } else {
                    ranges[job + 1][i + 1]
                };
                ranges[job][i] = range_num;
                if new_left != previous_left || new_right != previous_right {
                    range_sizes.push((last_range_end, i));
                    last_range_end = i + 1;
                    range_num += 1;
                }
                previous_left = new_left;
                previous_right = new_right;
            }
        }

        let mut job_position: HashMap<usize, usize> = HashMap::new();

        for i in 0..jobs.len() {
            job_position.insert(jobs[i], i);
        }

        assert_eq!(range_sizes.len(), range_num);

        //println!("{:?}", ranges);
        //println!("{:?}", range_sizes);
        let out = RangeTable {
            ranges,
            job_position,
            range_sizes,
            makespan,
        };
        return out;
    }

    fn get_range_bound(&self, range_num: usize) -> (usize, usize) {
        return self.range_sizes[range_num];
    }
}

impl DynBDD {
    pub fn leq(
        jobs: &Vec<usize>,
        vars: &Vec<usize>,
        weights: &Vec<usize>,
        limit: usize,
        with_fur_nodes: bool,
        already_assigned: usize,
        range_table: &RangeTable,
        timeout: &Timeout,
    ) -> Option<DynBDD> {
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
        let mut nodes: Vec<DynNode> = vec![];

        let true_node: DynNode = DynNode {
            var: 0,
            aux_var: 0,
            left_child: 1,
            right_child: 1,
            job_num: usize::MAX,
            range: (0, usize::MAX),
            range_num: usize::MAX,
        };
        let false_node = DynNode {
            var: 0,
            aux_var: 0,
            left_child: 0,
            right_child: 0,
            job_num: usize::MAX,
            range: (0, usize::MAX),
            range_num: usize::MAX,
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
                    let new_node: DynNode = DynNode {
                        var: vars[i + 1],
                        aux_var: 0,
                        left_child: prev_reachable_i_nodes[reachable_i],
                        right_child: if reachable_i + weights[i + 1] < prev_reachable_i_nodes.len()
                        {
                            prev_reachable_i_nodes[reachable_i + weights[i + 1]]
                        } else {
                            0
                        },
                        job_num: jobs[i + 1],
                        range: (reachable_i, reachable_i),
                        range_num: range_table.get_range(jobs[i + 1], reachable_i).unwrap(),
                    };
                    //let (range_lower, range_upper) = range_table.get_range_bound(new_node.range_num);
                    //println!("current reachable is {} but selected range is {} {} ", reachable_i, range_lower, range_upper);
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
        nodes.push(DynNode {
            var: vars[0],
            aux_var: 0,
            left_child: reachable_i_nodes[already_assigned + 0],
            right_child: reachable_i_nodes[already_assigned + weights[0]],
            job_num: jobs[0],
            range: (already_assigned, already_assigned),
            range_num: range_table.get_range(jobs[0], already_assigned).unwrap(),
        });

        return Some(DynBDD { nodes: nodes });
    }

    pub fn assign_aux_vars(&mut self, var_name_generator: &mut VarNameGenerator) {
        for i in (0..self.nodes.len()).rev() {
            if self.nodes[i].aux_var == 0 {
                self.nodes[i].aux_var = var_name_generator.next();
            }
        }
    }

    pub fn encode(&self) -> Clauses {
        let mut clauses: Clauses = Clauses::new();

        // we first have to handle that false and true nodes, as well as the root node
        clauses.add_clause(Clause {
            vars: vec![-(self.nodes[0].aux_var as i32)],
        });
        clauses.add_clause(Clause {
            vars: vec![self.nodes[1].aux_var as i32],
        });
        clauses.add_clause(Clause {
            vars: vec![self.nodes[self.nodes.len() - 1].aux_var as i32],
        });
        // now we handle the two clauses per node
        for i in 2..self.nodes.len() {
            // the first clause is -left_child_aux -> -node_aux
            clauses.add_clause(Clause {
                vars: vec![
                    // this is correct but makes it slower for some reason
                    (self.nodes[i].var as i32),
                    -(self.nodes[i].aux_var as i32),
                    (self.nodes[self.nodes[i].left_child].aux_var as i32),
                ],
            });
            // the second clause is -right_child_aux & node_var -> -node_aux

            clauses.add_clause(Clause {
                vars: vec![
                    -(self.nodes[i].var as i32),
                    -(self.nodes[i].aux_var as i32),
                    (self.nodes[self.nodes[i].right_child].aux_var as i32),
                ],
            });
        }

        return clauses;
    }

    pub fn encode_bdd_bijective_relation(&self, bdd2: &DynBDD) -> Clauses {
        // bijection says that the node i in bdd1 is equivalent to the variable bijection[i] in bdd2
        let mut bijection: Vec<usize> = vec![0, 1];
        let mut bdd2_i = 2;

        for bdd1_i in 2..self.nodes.len() {
            while bdd2_i != bdd2.nodes.len()
                && (bdd2.nodes[bdd2_i].job_num < self.nodes[bdd1_i].job_num
                    || (bdd2.nodes[bdd2_i].range_num < self.nodes[bdd1_i].range_num))
            {
                bdd2_i += 1;
            }

            if bdd2_i != bdd2.nodes.len()
                && bdd2.nodes[bdd2_i].job_num == self.nodes[bdd1_i].job_num
                && bdd2.nodes[bdd2_i].range_num == self.nodes[bdd1_i].range_num
            {
                bijection.push(bdd2_i);
                bdd2_i += 1;
            } else {
                bijection.push(usize::MAX);
            }
        }

        let mut clauses = Clauses::new();
        for bdd1_node in 2..self.nodes.len() {
            if bijection[bdd1_node] == usize::MAX {
                continue;
            }
            let node: &DynNode = &self.nodes[bdd1_node];
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

    pub fn get_fur_vars(
        &self,
        range_table: &RangeTable,
        solution: &PartialSolution,
    ) -> Vec<(usize, usize)> {
        let mut jobs_in_bdd = vec![];
        for i in &self.nodes {
            if i.job_num != usize::MAX
                && (jobs_in_bdd.len() == 0 || jobs_in_bdd[jobs_in_bdd.len() - 1] != i.job_num)
            {
                jobs_in_bdd.push(i.job_num);
            }
        }

        let mut out: Vec<(usize, usize)> = vec![];

        for i in 0..jobs_in_bdd.len() {
            let job = jobs_in_bdd[i];
            let next_job = if i != jobs_in_bdd.len() - 1 {
                jobs_in_bdd[i + 1]
            } else {
                usize::MAX
            };
            let fur_val = range_table.makespan - solution.instance.job_sizes[job];
            for node in &self.nodes {
                if node.job_num == usize::MAX {
                    continue;
                }
                if node.job_num == next_job {
                    break;
                }
                let (_, upper) = node.range;
                let (_, range_upper) = range_table.get_range_bound(node.range_num);

                if fur_val == upper || fur_val == range_upper {
                    //println!("job size: {} range {} {}, node range {} {}", solution.instance.job_sizes[job], range_lower, range_upper, lower, upper);
                    out.push((job, node.aux_var));
                }
            }
        }

        return out;
    }
}
