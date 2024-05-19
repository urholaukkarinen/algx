use std::collections::{BTreeSet, VecDeque};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct SolverBuilder {
    rows: Vec<Vec<usize>>,
    initial_columns: Vec<usize>,
}

#[wasm_bindgen]
impl SolverBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, row: Vec<usize>) {
        self.rows.push(row);
    }

    pub fn set_initial_columns(&mut self, initial_columns: Vec<usize>) {
        self.initial_columns = initial_columns;
    }

    pub fn build(self) -> Solver {
        Solver {
            solver: crate::Solver::new(self.rows, self.initial_columns),
        }
    }
}

#[wasm_bindgen]
pub struct Solver {
    solver: crate::Solver,
}

#[wasm_bindgen]
impl Solver {
    pub fn next_solution(&mut self) -> Array {
        let next_solution = self.solver.next();
        into_js_array(next_solution.unwrap_or_default())
    }

    pub async fn all_solutions(self) -> Array {
        self.solver.map(into_js_array).collect()
    }
}

fn into_js_array<T>(vec: Vec<T>) -> Array
where
    JsValue: From<T>,
{
    vec.into_iter().map(JsValue::from).collect()
}

#[wasm_bindgen]
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[wasm_bindgen]
pub fn generate_polyamino_rows(square_count: usize) -> Array {
    let mut shapes = BTreeSet::new();

    let mut stack: VecDeque<Vec<(i32, i32)>> = VecDeque::new();
    stack.push_back(vec![(0, 0)]);

    while let Some(shape) = stack.pop_front() {
        if shape.len() == square_count {
            let mut ret = vec![];
            let min_x = shape.iter().map(|(x, _)| x).min().copied().unwrap();
            let min_y = shape.iter().map(|(_, y)| y).min().copied().unwrap();

            for (x, y) in shape {
                ret.push(Pos::new(x - min_x, y - min_y));
            }

            shapes.insert(ret);
        } else {
            for (i, j) in [(1, 0), (0, 1), (0, -1), (-1, 0)] {
                let mut shape = shape.clone();
                let mut pos = shape.last().copied().unwrap();
                pos.0 += i;
                pos.1 += j;

                if !shape.contains(&pos) {
                    shape.push(pos);
                    stack.push_back(shape);
                }
            }
        }
    }

    into_js_array(shapes.into_iter().map(into_js_array).collect())
}
