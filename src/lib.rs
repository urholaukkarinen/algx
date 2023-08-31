///! Implementation of [https://en.wikipedia.org/wiki/Knuth%27s_Algorithm_X](Knuth's Algorithm X)
///! for solving the [https://en.wikipedia.org/wiki/Exact_cover](exact cover) problem.
///!
///!
use std::{collections::BTreeMap, ops::Deref, ops::DerefMut, pin::Pin, ptr::null_mut};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct NodeRef {
    inner: *mut Node,
}

impl NodeRef {
    fn new(inner: *mut Node) -> Self {
        Self { inner }
    }

    fn invalid() -> Self {
        Self { inner: null_mut() }
    }

    fn is_valid(&self) -> bool {
        !self.inner.is_null()
    }
}

impl Deref for NodeRef {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        if cfg!(debug_assertions) && !self.is_valid() {
            panic!("Tried to access invalid node!");
        }

        unsafe { &*self.inner }
    }
}

impl DerefMut for NodeRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if cfg!(debug_assertions) && !self.is_valid() {
            panic!("Tried to access invalid node!");
        }

        unsafe { &mut *self.inner }
    }
}

pub struct Node {
    left: NodeRef,
    right: NodeRef,
    up: NodeRef,
    down: NodeRef,
    header: NodeRef,

    row: isize,
    col: usize,
}

impl Node {
    fn new() -> Pin<Box<Self>> {
        let mut node = Box::pin(Node {
            left: NodeRef::invalid(),
            right: NodeRef::invalid(),
            up: NodeRef::invalid(),
            down: NodeRef::invalid(),
            header: NodeRef::invalid(),
            row: 0,
            col: 0,
        });

        let ptr = node.to_ptr();
        node.left = ptr;
        node.right = ptr;
        node.up = ptr;
        node.down = ptr;
        node.header = ptr;
        node
    }

    fn to_ptr(self: &Pin<Box<Self>>) -> NodeRef {
        NodeRef::new(self.as_ref().deref() as *const Self as *mut Self)
    }
}

struct Step {
    node: NodeRef,
    backtracking: bool,
}

pub struct Solver {
    _nodes: Vec<Pin<Box<Node>>>,
    header: NodeRef,

    column_sizes: Vec<usize>,
    step_stack: Vec<Step>,
    partial_solution: Vec<usize>,
}

impl Solver {
    /// Creates a new solver for given rows.
    /// Columns in the rows are assumed to be in ascending order
    pub fn new(rows: Vec<Vec<usize>>, partial_solution: Vec<usize>) -> Self {
        let column_count = rows.iter().flatten().copied().max().unwrap_or_default() + 1;

        let mut header_row: Vec<NodeRef> = vec![];
        let mut nodes: Vec<Pin<Box<Node>>> = vec![];

        let mut above_nodes = vec![NodeRef::invalid(); column_count];

        let mut column_sizes = vec![0; column_count];

        let mut columns_to_cover = BTreeMap::new();

        for (row_idx, row) in rows.into_iter().enumerate() {
            let mut first = NodeRef::invalid();
            let mut prev = NodeRef::invalid();

            for col_idx in row {
                let mut node = Node::new();
                node.row = row_idx as isize;
                node.col = col_idx;

                column_sizes[col_idx] += 1;

                let node_ptr = node.to_ptr();

                if !first.is_valid() {
                    first = node_ptr;
                }

                if prev.is_valid() {
                    link_horizontal(prev, node_ptr);
                }

                let above = &mut above_nodes[col_idx];
                if above.is_valid() {
                    node.up = *above;
                    node.down = above.down;
                    node.header = above.header;
                    node.header.up = node_ptr;
                    above.down = node_ptr;
                } else {
                    let mut header_node = Node::new();
                    header_node.row = -1;
                    header_node.col = col_idx;

                    let header_ptr = header_node.to_ptr();
                    header_row.push(header_ptr);

                    header_node.header = header_ptr;
                    header_node.up = node_ptr;
                    header_node.down = node_ptr;
                    node.up = header_ptr;
                    node.down = header_ptr;
                    node.header = header_ptr;

                    nodes.push(header_node);
                }

                above_nodes[col_idx] = node_ptr;
                prev = node_ptr;

                if partial_solution.contains(&col_idx) && !columns_to_cover.contains_key(&node.col)
                {
                    columns_to_cover.insert(node.col, node_ptr);
                }

                nodes.push(node);
            }

            if first.is_valid() && prev.is_valid() {
                link_horizontal(prev, first);
            }
        }

        header_row.sort_by(|a, b| a.col.cmp(&b.col));
        let mut first_header = header_row.iter().next().copied().unwrap();
        let mut last_header = header_row.iter().last().copied().unwrap();
        first_header.left = last_header;
        last_header.right = first_header;

        header_row
            .windows(2)
            .for_each(|nodes| link_horizontal(nodes[0], nodes[1]));

        let mut header_root = Node::new();
        let header_root_ptr = header_root.to_ptr();

        header_root.right = first_header;
        first_header.left = header_root_ptr;

        header_root.left = last_header;
        last_header.right = header_root_ptr;

        nodes.push(header_root);

        let mut solver = Self {
            _nodes: nodes,
            header: header_root_ptr,
            partial_solution: Vec::with_capacity(header_row.len()),
            column_sizes,
            step_stack: vec![],
        };

        for node in columns_to_cover.values() {
            let node = node.header.down;
            solver.cover(node);
        }

        if let Some(node) = solver.choose_column() {
            solver.step_stack.push(Step {
                node,
                backtracking: false,
            });
        }

        solver
    }

    pub fn is_completed(&self) -> bool {
        self.step_stack.is_empty()
    }

    fn choose_column(&self) -> Option<NodeRef> {
        let mut best_column = None;
        let mut best_size = usize::MAX;

        let mut current_node = self.header.right;

        while current_node != self.header {
            let current_size = self.column_sizes[current_node.col];
            if current_size < best_size {
                best_column = Some(current_node);
                best_size = current_size;
            }
            current_node = current_node.right;
        }

        best_column.map(|node| node.down)
    }

    pub fn step(&mut self) -> Option<Vec<usize>> {
        let Some(Step { node, backtracking }) = self.step_stack.pop() else {
            return None;
        };

        if node == node.header {
            return None;
        }

        if backtracking {
            self.step_backward(node);
        } else {
            self.step_forward(node);
        }

        if self.header.right == self.header {
            Some(self.partial_solution.clone())
        } else {
            None
        }
    }

    fn step_forward(&mut self, node: NodeRef) {
        self.partial_solution.push(node.row as _);

        let mut current = node;
        loop {
            self.cover(current);

            current = current.right;
            if current == node {
                break;
            }
        }

        self.step_stack.push(Step {
            node,
            backtracking: true,
        });

        if let Some(node) = self.choose_column() {
            self.step_stack.push(Step {
                node,
                backtracking: false,
            });
        }
    }

    fn step_backward(&mut self, node: NodeRef) {
        self.partial_solution.pop();

        let mut current = node.left;
        loop {
            self.uncover(current);

            if current == node {
                break;
            }
            current = current.left;
        }

        if node.down != node.header {
            self.step_stack.push(Step {
                node: node.down,
                backtracking: false,
            });
        }
    }

    fn cover(&mut self, node: NodeRef) {
        self.detach_column(node);

        let mut down = node.header.down;
        while down != node.header {
            self.detach_row(down);
            down = down.down;
        }
    }

    fn uncover(&mut self, node: NodeRef) {
        let mut up = node.header.up;
        while up != node.header {
            self.attach_row(up);
            up = up.up;
        }

        self.attach_column(node);
    }

    fn detach_row(&mut self, node: NodeRef) {
        let mut current = node.right;

        loop {
            if current == node {
                break;
            }

            self.column_sizes[current.col] -= 1;
            current.up.down = current.down;
            current.down.up = current.up;

            current = current.right;
        }
    }

    fn attach_row(&mut self, node: NodeRef) {
        let mut current = node.left;

        loop {
            if current == node {
                break;
            }

            self.column_sizes[current.col] += 1;
            current.down.up = current;
            current.up.down = current;

            current = current.left;
        }
    }

    fn detach_column(&self, mut node: NodeRef) {
        node.header.left.right = node.header.right;
        node.header.right.left = node.header.left;
    }

    fn attach_column(&self, mut node: NodeRef) {
        node.header.left.right = node.header;
        node.header.right.left = node.header;
    }
}

fn link_horizontal(mut left: NodeRef, mut right: NodeRef) {
    left.right = right;
    right.left = left;
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
