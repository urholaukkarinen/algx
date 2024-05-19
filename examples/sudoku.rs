#![allow(clippy::print_stdout)]

use algx::Solver;

fn create_sudoku_exact_cover_row(y: &usize, x: &usize, num: &usize) -> [usize; 4] {
    [
        y * 9 + x,
        (9 * 9) + (x * 9) + num,
        (9 * 9) * 2 + (y * 9) + num,
        (9 * 9) * 3 + (((y / 3) * 3 + (x / 3)) * 9 + num),
    ]
}

fn create_sudoku_exact_cover() -> Vec<Vec<usize>> {
    let mut rows = vec![];

    for y in 0..9 {
        for x in 0..9 {
            for num in 0..9 {
                rows.push(create_sudoku_exact_cover_row(&y, &x, &num).to_vec());
            }
        }
    }
    rows
}

fn x_y_num_from_row_index(i: usize) -> (usize, usize, usize) {
    let num = i % 9 + 1;
    let y = (i / 9) % 9;
    let x = i / (9 * 9);

    (x, y, num)
}

fn main() {
    let rows = create_sudoku_exact_cover();

    let solver = Solver::new(rows, vec![]);

    for solution in solver.skip(1).take(1) {
        let mut sudoku: [[u8; 9]; 9] = Default::default();
        for (x, y, num) in solution.into_iter().map(x_y_num_from_row_index) {
            sudoku[y][x] = num as u8;
        }

        println!("-------------------------");
        for rows in sudoku.chunks(3) {
            for row in rows {
                for col in row.chunks(3) {
                    print!("| ");
                    for num in col {
                        print!("{} ", num);
                    }
                }
                println!("|");
            }
            println!("-------------------------");
        }

        println!();
    }
}
