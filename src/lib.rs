//! Implementation of [Knuth's Algorithm X](https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X)
//! for solving the [exact cover](https://en.wikipedia.org/wiki/Exact_cover) problem.
//!
mod node;
#[cfg(target_arch = "wasm32")]
mod wasm;

use node::{Node, NodeId};

use std::collections::BTreeMap;

#[derive(Default, Debug, Clone)]
struct SolverState {
    nodes: Vec<Node>,
    header: NodeId,
    column_sizes: Vec<usize>,
}

impl SolverState {
    fn new_node(&mut self) -> NodeId {
        self.nodes.push(Node::default());
        NodeId::new(self.nodes.len() - 1)
    }

    fn link_horizontal(&mut self, left_id: NodeId, right_id: NodeId) {
        let left = self.node_mut(left_id);
        left.right = right_id;

        let right = self.node_mut(right_id);
        right.left = left_id;
    }

    fn detach_column(&mut self, node_id: NodeId) {
        let node = self.node(node_id);
        let header = self.node(node.header);

        let header_left_id = header.left;
        let header_right_id = header.right;

        let header_left = self.node_mut(header_left_id);
        header_left.right = header_right_id;

        let header_right = self.node_mut(header_right_id);
        header_right.left = header_left_id;
    }

    fn attach_column(&mut self, node_id: NodeId) {
        let node = self.node_mut(node_id);
        let header_id = node.header;

        let header = self.node_mut(header_id);
        let header_left_id = header.left;
        let header_right_id = header.right;

        let header_left = self.node_mut(header_left_id);
        header_left.right = header_id;

        let header_right = self.node_mut(header_right_id);
        header_right.left = header_id;
    }

    fn detach_row(&mut self, node_id: NodeId) {
        let mut current_id = self.node_mut(node_id).right;

        loop {
            if current_id == node_id {
                break;
            }

            let current_node = self.node_mut(current_id);
            let current_col_idx = current_node.col;
            let current_down_id = current_node.down;
            let current_up_id = current_node.up;
            let current_right_id = current_node.right;

            self.node_mut(current_up_id).down = current_down_id;
            self.node_mut(current_down_id).up = current_up_id;

            self.column_sizes[current_col_idx] -= 1;

            current_id = current_right_id;
        }
    }

    fn attach_row(&mut self, node_id: NodeId) {
        let mut current_id = self.node_mut(node_id).left;

        loop {
            if current_id == node_id {
                break;
            }

            let current_node = self.node_mut(current_id);
            let current_col_idx = current_node.col;
            let current_down_id = current_node.down;
            let current_left_id = current_node.left;
            let current_up_id = current_node.up;

            self.column_sizes[current_col_idx] += 1;

            self.node_mut(current_down_id).up = current_id;
            self.node_mut(current_up_id).down = current_id;

            current_id = current_left_id;
        }
    }

    fn node_column_size(&self, id: NodeId) -> usize {
        self.column_sizes[self.node(id).col]
    }

    fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.value()]
    }

    fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.value()]
    }

    fn header_node_mut(&mut self, id: NodeId) -> &mut Node {
        let header_node_id = self.node_mut(id).header;

        self.node_mut(header_node_id)
    }
}

#[derive(Debug, Copy, Clone)]
struct Step {
    node_id: NodeId,
    backtracking: bool,
}

#[derive(Debug, Default, Clone)]
pub struct Solver {
    state: SolverState,
    step_stack: Vec<Step>,
    partial_solution: Vec<usize>,
}

impl Solver {
    /// Creates a new solver for given rows. Columns in the rows are assumed to be in ascending order
    pub fn new(rows: Vec<Vec<usize>>, partial_solution: Vec<usize>) -> Self {
        let column_count = rows.iter().flatten().copied().max().unwrap_or_default() + 1;

        let mut state = SolverState {
            nodes: vec![],
            header: Default::default(),
            column_sizes: vec![0; column_count],
        };

        let mut header_row: Vec<NodeId> = vec![];

        let mut above_nodes = vec![NodeId::invalid(); column_count];

        let mut columns_to_cover = BTreeMap::new();

        for (row_idx, row) in rows.into_iter().enumerate() {
            let mut first = NodeId::invalid();
            let mut prev = NodeId::invalid();

            for col_idx in row {
                let node_id = state.new_node();

                state.node_mut(node_id).row = row_idx as isize;
                state.node_mut(node_id).col = col_idx;

                state.column_sizes[col_idx] += 1;

                if !first.is_valid() {
                    first = node_id;
                }

                if prev.is_valid() {
                    state.link_horizontal(prev, node_id);
                }

                let above_id = above_nodes[col_idx];
                if above_id.is_valid() {
                    let above_node = state.node_mut(above_id);
                    let above_down_id = above_node.down;
                    let above_header_id = above_node.header;

                    above_node.down = node_id;

                    let node = state.node_mut(node_id);
                    node.up = above_id;
                    node.down = above_down_id;
                    node.header = above_header_id;

                    state.header_node_mut(node_id).up = node_id;
                } else {
                    let header_id = state.new_node();
                    header_row.push(header_id);

                    let header = state.node_mut(header_id);
                    header.row = -1;
                    header.col = col_idx;
                    header.header = header_id;
                    header.up = node_id;
                    header.down = node_id;

                    let node = state.node_mut(node_id);
                    node.up = header_id;
                    node.down = header_id;
                    node.header = header_id;
                }

                above_nodes[col_idx] = node_id;
                prev = node_id;

                if partial_solution.contains(&col_idx) && !columns_to_cover.contains_key(&col_idx) {
                    columns_to_cover.insert(col_idx, node_id);
                }
            }

            if first.is_valid() && prev.is_valid() {
                state.link_horizontal(prev, first);
            }
        }

        header_row.sort_by(|a, b| {
            let a_col = state.node_mut(*a).col;
            let b_col = state.node_mut(*b).col;
            a_col.cmp(&b_col)
        });

        let Some(first_header_id) = header_row.first().copied() else {
            return Default::default();
        };

        let last_header_id = header_row.iter().last().copied().unwrap_or(first_header_id);

        state.node_mut(first_header_id).left = last_header_id;
        state.node_mut(last_header_id).right = first_header_id;

        header_row.windows(2).for_each(|nodes| {
            state.link_horizontal(nodes[0], nodes[1]);
        });

        let header_root_id = state.new_node();

        state.node_mut(header_root_id).right = first_header_id;
        state.node_mut(first_header_id).left = header_root_id;

        state.node_mut(header_root_id).left = last_header_id;
        state.node_mut(last_header_id).right = header_root_id;

        state.header = header_root_id;

        let mut solver = Self {
            state: state.clone(),
            partial_solution: Vec::with_capacity(header_row.len()),
            step_stack: vec![],
        };

        for column_node_id in columns_to_cover.values() {
            let column_first_node_id = state.header_node_mut(*column_node_id).down;
            solver.cover(column_first_node_id);
        }

        if let Some(node_id) = solver.choose_column() {
            solver.step_stack.push(Step {
                node_id,
                backtracking: false,
            });
        }

        solver
    }

    fn choose_column(&self) -> Option<NodeId> {
        let mut best_column_id = None;
        let mut best_size = usize::MAX;

        let mut current_node_id = self.state.node(self.state.header).right;

        while current_node_id != self.state.header {
            let current_size = self.state.node_column_size(current_node_id);

            if current_size < best_size {
                best_column_id = Some(current_node_id);
                best_size = current_size;
            }
            current_node_id = self.state.node(current_node_id).right;
        }

        Some(self.state.node(best_column_id?).down)
    }

    pub fn partial_solution(&self) -> &[usize] {
        &self.partial_solution
    }

    pub fn is_completed(&self) -> bool {
        self.step_stack.is_empty()
    }

    fn cover(&mut self, node_id: NodeId) {
        self.state.detach_column(node_id);

        let node = self.state.node_mut(node_id);
        let node_header_id = node.header;

        let mut down_id = self.state.node_mut(node_header_id).down;
        while down_id != node_header_id {
            self.state.detach_row(down_id);

            down_id = self.state.node_mut(down_id).down;
        }
    }

    fn uncover(&mut self, node_id: NodeId) {
        let node_header_id = self.state.node(node_id).header;
        let mut up_id = self.state.node(node_header_id).up;

        while up_id != node_header_id {
            self.state.attach_row(up_id);
            up_id = self.state.node(up_id).up;
        }

        self.state.attach_column(node_id);
    }

    pub fn step(&mut self) -> Option<Vec<usize>> {
        let Step {
            node_id,
            backtracking,
        } = self.step_stack.pop()?;

        let node_header_id = self.state.node(node_id).header;

        if node_id == node_header_id {
            return None;
        }

        if backtracking {
            self.step_backward(node_id);
        } else {
            self.step_forward(node_id);
        }

        let header_root_id = self.state.header;

        if self.state.node_mut(header_root_id).right == header_root_id {
            Some(self.partial_solution.clone())
        } else {
            None
        }
    }

    fn step_forward(&mut self, node_id: NodeId) {
        let node_row = self.state.node(node_id).row;
        self.partial_solution.push(node_row as _);

        let mut current_id = node_id;
        loop {
            self.cover(current_id);

            current_id = self.state.node(current_id).right;
            if current_id == node_id {
                break;
            }
        }

        self.step_stack.push(Step {
            node_id,
            backtracking: true,
        });

        if let Some(node_id) = self.choose_column() {
            self.step_stack.push(Step {
                node_id,
                backtracking: false,
            });
        }
    }

    fn step_backward(&mut self, node_id: NodeId) {
        self.partial_solution.pop();

        let mut current_id = self.state.node(node_id).left;
        loop {
            self.uncover(current_id);

            if current_id == node_id {
                break;
            }
            current_id = self.state.node(current_id).left;
        }

        let node_down = self.state.node(node_id).down;
        let node_header = self.state.node(node_id).header;

        if node_down != node_header {
            self.step_stack.push(Step {
                node_id: node_down,
                backtracking: false,
            });
        }
    }
}

impl Iterator for Solver {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.is_completed() {
            let step = self.step();

            if step.is_some() {
                return step;
            }
        }

        None
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;

    #[test]
    fn test_basic_solve() {
        // [x, x, -, -]
        // [x, -, x, -]
        // [-, x, -, x]
        // [-, -, x, x]
        // [x, x, x, -]
        // [-, x, x, x]
        //
        //  __
        // |# |
        // |# |
        //  ""
        let solver = Solver::new(vec![
            vec![0, 1],
            vec![0, 2],
            vec![1, 3],
            vec![2, 3],
            vec![0, 1, 2],
            vec![1, 2, 3],
        ], vec![0, 2]);

        let solutions = solver.collect::<Vec<_>>();

        assert_eq!(vec![vec![2]], solutions);
    }
}
